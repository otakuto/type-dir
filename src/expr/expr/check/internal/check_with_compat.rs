#[cfg(test)]
#[path = "check_with_compat_tests/tests.rs"]
mod tests;

use std::collections::HashSet;

use indexmap::IndexMap;

use super::id_shape::{ChildRef, IdShape};
use super::id_shape_derive::{derive_rule_id_shape, resolve_rule_path_shape};
use crate::error::SemanticError;
use crate::expr::template_ref::ns_head_id;
use crate::yaml::{RuleName, WithShape, YamlEntry, YamlEntryKind, YamlRule};

/// Checks caller-side with declaration shape compatibility (spec 5 validation B / E018).
///
/// For `RuleType { rule, path }` declarations: the passed value must be a bare `${id}` whose
/// shape is subsumable into the shape derived by following `path` from rule `rule`. Mismatch
/// produces `WithShapeMismatch`. An undefined rule referenced in `RuleType` produces
/// `UndefinedShapeRule` (E021). The check is skipped when the declaration is Scalar or when the
/// passed value is not a bare `${id}`.
///
/// `id_shapes` is the global id → static shape map built once by the caller (via `build_id_shapes`)
/// and shared with `check_with_shapes` to avoid redundant full-AST traversal.
pub fn check_with_compat(
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        // Check E021: RuleType references to undefined rules in this rule's with_params
        for (with_var, yaml_shape) in &rule.with_params {
            if let Ok(WithShape::RuleType { rule: ref_rule, .. }) = yaml_shape.to_shape()
                && !rules.contains_key(&ref_rule)
            {
                errors.push(SemanticError::UndefinedShapeRule {
                    rule: rule_name.0.clone(),
                    with: with_var.0.clone(),
                    ref_rule: ref_rule.0.clone(),
                });
            }
        }
        for entry in &rule.body {
            check_entry(entry, rules, id_shapes, &HashSet::new(), &mut errors);
        }
    }
    errors
}

/// Checks a literal `with` arg value (no `${...}`) against its declared type, returning a mismatch
/// detail string when incompatible, or `None` when the literal satisfies the declared type.
///
/// Literals are scalar (string/number/bool) — composite literals are rejected at parse time. The
/// literal's type is inferred lexically: an integer/float text is number-compatible, `true`/`false`
/// is bool-compatible, and any text is string-compatible.
/// - `String`: always compatible (every scalar literal is a valid string).
/// - `Number`: compatible iff the text parses as an `f64`.
/// - `Bool`: compatible iff the text is exactly `true` or `false`.
/// - `Array` / `Object` / `RuleType`: a scalar literal cannot satisfy a composite/rule type.
/// - `Scalar` (permissive fallback): always compatible.
fn literal_type_mismatch(value: &str, decl: &WithShape) -> Option<String> {
    match decl {
        WithShape::Scalar | WithShape::String => None,
        WithShape::Number => {
            if value.parse::<f64>().is_ok() {
                None
            } else {
                Some(format!(
                    "literal `{value}` is not a number but the declared type is `type.number`"
                ))
            }
        }
        WithShape::Bool => {
            if matches!(value, "true" | "false") {
                None
            } else {
                Some(format!(
                    "literal `{value}` is not a bool but the declared type is `type.bool`"
                ))
            }
        }
        WithShape::Array(_) => Some(format!(
            "literal `{value}` is a scalar but the declared type is an array; \
             pass a list by reference (`value: ${{...}}`)"
        )),
        WithShape::Object(_) => Some(format!(
            "literal `{value}` is a scalar but the declared type is an object"
        )),
        WithShape::RuleType { rule, .. } => Some(format!(
            "literal `{value}` is a scalar but the declared type is the rule type `rule.{}`",
            rule.0
        )),
    }
}

/// Recursively checks entries and compares splice input values against their declared shapes.
///
/// `bound_vars` holds the for-loop lexical variable names in scope at the current position. A bare
/// `${var}` whose name is bound is a lexical variable, not a global id producer, so its shape
/// compatibility is checked in `check_with_shapes` (via the for-source shape) rather than here.
/// Skipping such names avoids false positives when a for variable happens to collide with a global
/// id name (which would otherwise be looked up in `id_shapes` and compared against an unrelated shape).
fn check_entry(
    entry: &YamlEntry,
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
    bound_vars: &HashSet<String>,
    errors: &mut Vec<SemanticError>,
) {
    match &entry.kind {
        YamlEntryKind::Use {
            rule: target,
            with_args,
            ..
        } => {
            // Check the with args of a use (bare rule reference)
            if let Some(target_rule) = rules.get(target) {
                for (with_var, value) in with_args {
                    let Some(yaml_shape) = target_rule.with_params.get(with_var) else {
                        continue; // Undeclared keys are handled by check_with_keys
                    };
                    let Ok(decl_shape) = yaml_shape.to_shape() else {
                        continue; // Unparseable shapes are handled by check_with_shapes
                    };
                    // A value with no `${...}` is a literal; check its lexical type against the
                    // declared type. References (`${...}`) are checked via the shape path below.
                    if !value.contains("${") {
                        if let Some(detail) = literal_type_mismatch(value, &decl_shape) {
                            errors.push(SemanticError::WithShapeMismatch {
                                rule: target.0.clone(),
                                with: with_var.0.clone(),
                                detail,
                            });
                        }
                        continue;
                    }
                    // Only RuleType declarations are checked here; Scalar params are not targeted.
                    if let WithShape::RuleType {
                        rule: ref_rule,
                        path,
                    } = &decl_shape
                    {
                        // Only compare when the passed value is a single-segment namespaced id
                        // reference (`${dir.id}` / `${file.id}` / ...).
                        let Some(id) = ns_head_id(value) else {
                            continue;
                        };
                        if bound_vars.contains(id) {
                            // for-bound lexical variable: not a global id producer; its shape is
                            // checked in check_with_shapes via the for-source shape.
                            continue;
                        }
                        let Some(id_shape) = id_shapes.get(id) else {
                            continue;
                        };
                        // Derive the expected shape by following the declared path from the rule
                        if let Some(expected) = resolve_rule_path_shape(ref_rule, path, rules) {
                            let mut visited = HashSet::new();
                            if !subsumes(&expected, id_shape, rules, &mut visited) {
                                errors.push(SemanticError::WithShapeMismatch {
                                    rule: target.0.clone(),
                                    with: with_var.0.clone(),
                                    detail: format!(
                                        "passed id `{id}` shape does not match the expected shape of rule `{}`",
                                        ref_rule.0
                                    ),
                                });
                            }
                        }
                        // Undefined RuleType is reported separately (E021 in the declaring rule)
                    }
                }
            }
        }
        YamlEntryKind::Choice { body, .. } => {
            for alt in body {
                check_entry(alt, rules, id_shapes, bound_vars, errors);
            }
        }
        YamlEntryKind::Dir { body: children, .. } | YamlEntryKind::File { body: children, .. } => {
            if let Some(children) = children {
                for child in children {
                    check_entry(child, rules, id_shapes, bound_vars, errors);
                }
            }
        }
        YamlEntryKind::Group { body: children, .. } => {
            for child in children {
                check_entry(child, rules, id_shapes, bound_vars, errors);
            }
        }
        YamlEntryKind::For {
            var,
            body: for_rules,
            ..
        } => {
            // Add the for variable to the bound set so its name is not mistaken for a global id.
            let mut inner = bound_vars.clone();
            inner.insert(var.0.clone());
            for child in for_rules {
                check_entry(child, rules, id_shapes, &inner, errors);
            }
        }
        YamlEntryKind::Match { body, .. } => {
            for arm in body {
                check_entry(arm, rules, id_shapes, bound_vars, errors);
            }
        }
        YamlEntryKind::Fetch { body } => {
            for alt in body {
                check_entry(alt, rules, id_shapes, bound_vars, errors);
            }
        }
        // a value binding passes no `with:` args and owns no children
        YamlEntryKind::Value { .. } => {}
    }
}

/// Coinductive width subsumption: checks whether `actual` is subsumable into `expected`.
///
/// `expected` is the shape of the rule's public id; `actual` is the shape of the passed id.
/// Rules:
/// - `RuleRef(N) vs RuleRef(M)`: compatible iff N == M (stop immediately).
/// - `RuleRef(N) vs Inline(shape)` or vice versa: expand RuleRef one level and recurse.
/// - `Inline(exp) vs Inline(act)`: check captures and child_ids recursively.
/// - `visited` prevents infinite recursion under coinductive interpretation.
///
/// Returns `true` when the shapes are compatible.
fn subsumes(
    expected: &IdShape,
    actual: &IdShape,
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<(String, String)>,
) -> bool {
    // Check captures: all captures in expected must be in actual
    for cap in &expected.captures {
        if !actual.captures.contains(cap) {
            return false;
        }
    }
    // Check child_ids: all children in expected must be in actual with compatible refs
    for (child_name, expected_ref) in &expected.child_ids {
        let Some(actual_ref) = actual.child_ids.get(child_name) else {
            return false;
        };
        if !child_ref_subsumes(expected_ref, actual_ref, rules, visited) {
            return false;
        }
    }
    true
}

/// Checks compatibility between two `ChildRef` values.
fn child_ref_subsumes(
    expected: &ChildRef,
    actual: &ChildRef,
    rules: &IndexMap<RuleName, YamlRule>,
    visited: &mut HashSet<(String, String)>,
) -> bool {
    match (expected, actual) {
        // RuleRef vs RuleRef: name equality stops recursion
        (ChildRef::RuleRef(n), ChildRef::RuleRef(m)) => n == m,
        // Inline vs Inline: recurse into sub-shapes
        (ChildRef::Inline(exp_shape), ChildRef::Inline(act_shape)) => {
            subsumes(exp_shape, act_shape, rules, visited)
        }
        // RuleRef vs Inline: expand RuleRef to its derived shape and recurse
        (ChildRef::RuleRef(n), ChildRef::Inline(act_shape)) => {
            let mut sorted_caps: Vec<&String> = act_shape.captures.iter().collect();
            sorted_caps.sort();
            let key = (n.0.clone(), format!("inline:{:?}", sorted_caps));
            if !visited.insert(key) {
                return true; // Coinductive: assume compatible if already visiting
            }
            if let Some(exp_shape) = derive_rule_id_shape(n, rules) {
                subsumes(&exp_shape, act_shape, rules, visited)
            } else {
                false
            }
        }
        // Inline vs RuleRef: expand RuleRef and recurse
        (ChildRef::Inline(exp_shape), ChildRef::RuleRef(m)) => {
            let mut sorted_caps: Vec<&String> = exp_shape.captures.iter().collect();
            sorted_caps.sort();
            let key = (format!("inline:{:?}", sorted_caps), m.0.clone());
            if !visited.insert(key) {
                return true;
            }
            if let Some(act_shape) = derive_rule_id_shape(m, rules) {
                subsumes(exp_shape, &act_shape, rules, visited)
            } else {
                false
            }
        }
    }
}
