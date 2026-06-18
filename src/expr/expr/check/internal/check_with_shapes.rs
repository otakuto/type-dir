#[cfg(test)]
#[path = "check_with_shapes_tests/tests.rs"]
mod tests;

use std::collections::HashSet;

use indexmap::IndexMap;

use super::check_rule_var_scope::collect_self_owned_ids;
use super::id_shape::{ChildRef, IdShape, NodeKind};
use super::id_shape_derive::{derive_rule_id_shape, resolve_rule_path_shape};
use super::pattern_util::{named_captures, pattern_str};
use crate::error::SemanticError;
use crate::expr::template_ref::extract_refs;
use crate::expr::{Hop, RefHead, parse_ref};
use crate::yaml::{
    RuleName, ValueExpr, WithShape, YamlEntry, YamlEntryKind, YamlForSource, YamlRule,
};

/// Validates the `with` shape declarations of each rule (spec 5 validation A + shape parsing).
///
/// 1. Shape parsing: if `YamlWithShape.to_shape()` fails, report `InvalidPattern`.
/// 2. Validation A (intra-rule consistency): check that all `${head.hops...}` references in
///    the rule body are valid against the statically-known shape of `head`. This now covers:
///    - With variables (Scalar / Records / RuleType)
///    - Self-owned ids (entries with an `id:` field in the rule body)
///    - For-binding variables (derived progressively during the walk)
///    - Captures introduced by ancestor entries (added during the walk)
///
///   All hops of a reference are followed (multi-hop), not just the first.
///
/// `id_shapes` is the global id → static shape map built once by the caller (via `build_id_shapes`)
/// and shared with `check_with_compat` to avoid redundant full-AST traversal.
pub fn check_with_shapes(
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
) -> Vec<SemanticError> {
    let mut errors = Vec::new();

    for (rule_name, rule) in rules {
        // 1. Shape-parse check + initial shape environment construction.
        let mut env: ShapeEnv = IndexMap::new();

        // Add each declared with param to the environment.
        for (var, yaml_shape) in &rule.with_params {
            match yaml_shape.to_shape() {
                Ok(shape) => {
                    env.insert(
                        var.0.clone(),
                        ShapeEntry {
                            shape: shape_of_input(&shape, rules),
                            origin: var.0.clone(),
                        },
                    );
                }
                Err(e) => errors.push(SemanticError::InvalidPattern {
                    context: format!("rule `{}` with `{}`", rule_name.0, var.0),
                    reason: e.0,
                }),
            }
        }

        // Add self-owned ids to the environment.
        let mut self_owned: HashSet<String> = HashSet::new();
        collect_self_owned_ids(&rule.body, &mut self_owned);
        for id_name in &self_owned {
            if let Some(id_shape) = id_shapes.get(id_name) {
                env.insert(
                    id_name.clone(),
                    ShapeEntry {
                        shape: Shape::Id(id_shape.clone()),
                        origin: id_name.clone(),
                    },
                );
            }
            // If the id has no resolvable shape (e.g. splice+id from an ambiguous rule), skip.
        }

        // 2. Walk the rule body, validating all references against the environment.
        walk(
            &rule_name.0,
            &rule.body,
            &env,
            rules,
            id_shapes,
            &mut errors,
        );
    }
    errors
}

// ---------------------------------------------------------------------------
// Shape representation
// ---------------------------------------------------------------------------

/// The statically-known shape of a binding variable at a given point in a rule.
#[derive(Debug, Clone)]
enum Shape {
    /// A scalar value (capture result or a scalar input). No projection is valid.
    Scalar,
    /// Free-form / unknown shape. All hop checks are skipped (lenient fallback).
    FreeForm,
    /// An id-bearing entry shape with captures and typed child ids.
    Id(IdShape),
    /// An array type `[T]`. Direct field projection is invalid; iterate with `for` to reach the
    /// element shape.
    Array(Box<Shape>),
    /// An object type `{field: T, ...}`. `${x.field}` projects to the field's shape.
    Object(IndexMap<String, Shape>),
}

/// An entry in the shape environment: a shape paired with the origin label used in
/// error messages. For declared with params the origin is the with variable name; for
/// for-bindings it is inherited from the for-source's origin so that errors report the
/// original declared with param rather than the intermediate binding variable.
#[derive(Debug, Clone)]
struct ShapeEntry {
    shape: Shape,
    /// The declared-with name to use in `WithShapeMismatch.with` error fields.
    origin: String,
}

/// Maps each in-scope binding variable name to its shape and error-reporting origin.
type ShapeEnv = IndexMap<String, ShapeEntry>;

/// Converts a `WithShape` declaration into a `Shape` for the environment.
fn shape_of_input(with: &WithShape, rules: &IndexMap<RuleName, YamlRule>) -> Shape {
    match with {
        // Primitives and the permissive-scalar fallback all behave as a non-projectable scalar.
        WithShape::Scalar | WithShape::String | WithShape::Number | WithShape::Bool => {
            Shape::Scalar
        }
        WithShape::Array(elem) => Shape::Array(Box::new(shape_of_input(elem, rules))),
        WithShape::Object(fields) => Shape::Object(
            fields
                .iter()
                .map(|(name, ty)| (name.0.clone(), shape_of_input(ty, rules)))
                .collect(),
        ),
        WithShape::RuleType { rule, path } => {
            match resolve_rule_path_shape(rule, path, rules) {
                Some(id_shape) => Shape::Id(id_shape),
                // Undefined, ambiguous, or non-navigable path: treat as free-form (lenient;
                // other checks report undefined rules).
                None => Shape::FreeForm,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Walk
// ---------------------------------------------------------------------------

/// Recursively validates all `${...}` references in the entry list against `env`.
///
/// `env` maps every binding variable name in scope at the current position to its
/// static shape and origin label. It is cloned and extended as the walk descends into
/// children (captures from the entry's own pattern) and for-loop bodies (the iter variable).
fn walk(
    rule: &str,
    entries: &[YamlEntry],
    env: &ShapeEnv,
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
    errors: &mut Vec<SemanticError>,
) {
    for entry in entries {
        // Check references in the entry's own pattern and input values.
        // Filter to references that have any navigable hops (dotted or namespace-prefixed).
        for text in pattern_and_input_texts(entry) {
            for key in extract_refs(&text).into_iter().filter(|k| k.contains('.')) {
                let parsed = parse_ref(&key);
                validate_ref(
                    rule,
                    &key,
                    &parsed.head,
                    &parsed.hops,
                    env,
                    rules,
                    id_shapes,
                    errors,
                );
            }
        }

        // Build a child environment that adds captures introduced by this entry's pattern.
        let mut child_env = env.clone();
        if let YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. } =
            &entry.kind
        {
            for cap in named_captures(pattern) {
                child_env.insert(
                    cap.clone(),
                    ShapeEntry {
                        shape: Shape::Scalar,
                        origin: cap,
                    },
                );
            }
        }

        // Recurse based on kind.
        match &entry.kind {
            YamlEntryKind::Choice { body, .. } => {
                walk(rule, body, &child_env, rules, id_shapes, errors);
            }
            YamlEntryKind::Dir { body: inline, .. } | YamlEntryKind::File { body: inline, .. } => {
                if let Some(inline) = inline {
                    walk(rule, inline, &child_env, rules, id_shapes, errors);
                }
            }
            YamlEntryKind::Group { body: inline, .. } => {
                walk(rule, inline, &child_env, rules, id_shapes, errors);
            }
            YamlEntryKind::For {
                var,
                source,
                body: for_rules,
            } => {
                let (elem_shape, elem_origin) =
                    resolve_for_source_shape(rule, source, &child_env, rules, id_shapes, errors);
                let mut for_env = child_env.clone();
                for_env.insert(
                    var.0.clone(),
                    ShapeEntry {
                        shape: elem_shape,
                        origin: elem_origin,
                    },
                );
                walk(rule, for_rules, &for_env, rules, id_shapes, errors);
            }
            YamlEntryKind::Match {
                scrutinee: scrutinee_tmpl,
                body: arms,
            } => {
                // Recurse into match arm rules with per-arm narrowing when the scrutinee is a Sum id.
                //
                // When `match: ${c}` dispatches on a Sum id, each arm `- id: tag` constrains the
                // visible captures of `c` to those declared by the `tag` alternative only. This prevents
                // false-positive acceptance of cross-arm capture references (e.g. `${c.regex.cfg}` inside
                // the `service` arm where only `svc` is declared). If the scrutinee is not a Sum or the
                // arm has no id, fall back to the unnarrowed environment (lenient).
                let scrutinee_var = scrutinee_tmpl
                    .strip_prefix("${")
                    .and_then(|s| s.strip_suffix('}'))
                    .filter(|s| !s.contains('.') && !s.contains('}'))
                    .unwrap_or(scrutinee_tmpl.as_str());
                let scrutinee_entry = child_env.get(scrutinee_var);
                for arm in arms {
                    let arm_env = match narrow_scrutinee_for_arm(scrutinee_entry, arm) {
                        Some(narrowed_shape) => {
                            let se = scrutinee_entry.expect("narrowing implies a scrutinee entry");
                            let mut narrowed = child_env.clone();
                            narrowed.insert(
                                scrutinee_var.to_string(),
                                ShapeEntry {
                                    shape: narrowed_shape,
                                    origin: se.origin.clone(),
                                },
                            );
                            narrowed
                        }
                        None => child_env.clone(),
                    };
                    walk(
                        rule,
                        std::slice::from_ref(arm),
                        &arm_env,
                        rules,
                        id_shapes,
                        errors,
                    );
                }
            }
            YamlEntryKind::Fetch { body } => {
                walk(rule, body, &child_env, rules, id_shapes, errors);
            }
            YamlEntryKind::Use { .. } => {}
            // A value binding owns no children; its value templates were checked above.
            YamlEntryKind::Value { .. } => {}
        }
    }
}

/// Computes the narrowed scrutinee `Shape` for a single `match` arm.
///
/// Returns `Some(shape)` only when the scrutinee resolves to a Sum id (`sum_alts` present) and the
/// arm declares an id (the tag) that is one of the Sum's alternatives. The returned shape is the
/// per-alternative `IdShape`, restricting the visible captures/child ids to that alternative.
/// Returns `None` (no narrowing, fall back to the unnarrowed environment) in every other case:
/// non-Sum scrutinee, arm without an id, or an unknown tag.
fn narrow_scrutinee_for_arm(
    scrutinee_entry: Option<&ShapeEntry>,
    arm: &YamlEntry,
) -> Option<Shape> {
    let se = scrutinee_entry?;
    let arm_tag = arm.id.as_ref()?;
    let Shape::Id(id_shape) = &se.shape else {
        return None;
    };
    let alt_shape = id_shape.sum_alts.as_ref()?.get(&arm_tag.0)?;
    Some(Shape::Id(alt_shape.clone()))
}

// ---------------------------------------------------------------------------
// Reference validation
// ---------------------------------------------------------------------------

/// Validates a single reference against the shape environment using the new namespace grammar.
///
/// Head dispatch:
/// - `RefHead::Bare(name)`: look up `name` in `env`; follow `hops` via `step_shape` (for-iteration
///   variable shape inference; bare-reference rejection is handled separately by `check_rule_var_scope`).
/// - `RefHead::WithNs { param, tail }`: look up `param` in `env`; follow `tail` via `step_shape`.
/// - `RefHead::RuleNs { rule_id, tail }`: look up `rule_id` in `env`; follow `tail` via `step_shape`.
/// - `RefHead::UseNs { id, tail }`: look up `id` in `env`; follow `tail` via `step_shape`.
///   Treated identically to `RuleNs` for shape-walk purposes (auto-unwrap is a runtime concern).
/// - `RefHead::ValueNs { .. }`: `value:` bindings are scalars / string lists with no static record
///   shape, so there is nothing to validate here; the binding scope itself is checked by
///   `check_rule_var_scope`. Skip silently.
///
/// If the head is not in `env`, the reference is outside scope; `check_rule_var_scope` handles
/// that separately, so we silently skip here. Bare references (no hops/tail) need no shape validation.
#[allow(clippy::too_many_arguments)]
fn validate_ref(
    rule: &str,
    full_key: &str,
    parsed_head: &RefHead,
    hops: &[Hop],
    env: &ShapeEnv,
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
    errors: &mut Vec<SemanticError>,
) {
    // `use.<id>` and `for.<id>` resolve on the same path as `rule.<id>`: the wrapper is the head,
    // and the tail hops navigate into it explicitly (no auto-unwrap).
    let (env_key, effective_hops): (&str, &[Hop]) = match parsed_head {
        RefHead::Bare(name) => (name.as_str(), hops),
        RefHead::WithNs { param, tail } => (param.as_str(), tail.as_slice()),
        RefHead::RuleNs { rule_id, tail } => (rule_id.as_str(), tail.as_slice()),
        RefHead::UseNs { id, tail } => (id.as_str(), tail.as_slice()),
        RefHead::ForNs { id, tail } => (id.as_str(), tail.as_slice()),
        RefHead::FetchNs { id, tail } => (id.as_str(), tail.as_slice()),
        // Kind-qualified heads resolve on the same path as `for.<id>`: the head id is the env key
        // and the tail hops navigate into it. Strict kind matching is deferred to a later change.
        RefHead::DirNs { id, tail }
        | RefHead::FileNs { id, tail }
        | RefHead::GroupNs { id, tail }
        | RefHead::ChoiceNs { id, tail } => (id.as_str(), tail.as_slice()),
        // value bindings carry no record shape; nothing to validate against `env`.
        RefHead::ValueNs { .. } => return,
    };

    let Some(entry) = env.get(env_key) else {
        // Unknown head is handled by check_rule_var_scope.
        return;
    };
    if effective_hops.is_empty() {
        // No hops — no shape validation needed.
        return;
    }
    let origin = entry.origin.clone();
    let mut cur = entry.shape.clone();
    for hop in effective_hops {
        match step_shape(rule, full_key, &origin, &cur, hop, rules, id_shapes, errors) {
            Some(next) => cur = next,
            // step_shape already pushed an error; stop chaining.
            None => return,
        }
    }
}

/// Advances the shape by one hop, pushing an error and returning `None` on mismatch.
///
/// `origin` is the declared-input name used in error messages (not the intermediate
/// binding variable, which may differ for for-loop bindings).
#[allow(clippy::too_many_arguments)]
fn step_shape(
    rule: &str,
    full_key: &str,
    origin: &str,
    shape: &Shape,
    hop: &Hop,
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
    errors: &mut Vec<SemanticError>,
) -> Option<Shape> {
    match shape {
        Shape::FreeForm => {
            // No information; accept any hop (lenient).
            Some(Shape::FreeForm)
        }
        Shape::Scalar => {
            // Cannot project into a scalar.
            let hop_str = hop_display(hop);
            errors.push(SemanticError::WithShapeMismatch {
                rule: rule.to_string(),
                with: origin.to_string(),
                detail: format!("cannot project `.{hop_str}` on scalar `{origin}` in `{full_key}`"),
            });
            None
        }
        Shape::Id(id_shape) => step_id_shape(
            rule, full_key, origin, id_shape, hop, rules, id_shapes, errors,
        ),
        Shape::Array(_) => {
            // Cannot project directly into an array; it must be iterated with `for`.
            let hop_str = hop_display(hop);
            errors.push(SemanticError::WithShapeMismatch {
                rule: rule.to_string(),
                with: origin.to_string(),
                detail: format!(
                    "cannot project `.{hop_str}` on array `{origin}` in `{full_key}`; \
                     iterate it with `for` to access each element"
                ),
            });
            None
        }
        Shape::Object(fields) => match hop {
            Hop::Field(f) => match fields.get(f.as_str()) {
                Some(field_shape) => Some(field_shape.clone()),
                None => {
                    errors.push(SemanticError::WithShapeMismatch {
                        rule: rule.to_string(),
                        with: origin.to_string(),
                        detail: format!(
                            "object field `{f}` referenced via `{full_key}` is not declared in `{origin}`"
                        ),
                    });
                    None
                }
            },
            _ => {
                let hop_str = hop_display(hop);
                errors.push(SemanticError::WithShapeMismatch {
                    rule: rule.to_string(),
                    with: origin.to_string(),
                    detail: format!(
                        "cannot project `.{hop_str}` on object `{origin}` in `{full_key}`; \
                         use a field name (e.g. `${{{origin}.<field>}}`)"
                    ),
                });
                None
            }
        },
    }
}

/// Advances through an `IdShape` by one hop.
#[allow(clippy::too_many_arguments)]
fn step_id_shape(
    rule: &str,
    full_key: &str,
    origin: &str,
    id_shape: &IdShape,
    hop: &Hop,
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
    errors: &mut Vec<SemanticError>,
) -> Option<Shape> {
    match hop {
        Hop::Regex(g) => {
            // Positional groups ("0", "1", "2", ...) are always valid: they are not listed in
            // the static captures set (which only contains named groups), but are always present
            // at runtime when the regex matches.
            if g.chars().all(|c| c.is_ascii_digit()) || id_shape.captures.contains(g.as_str()) {
                Some(Shape::Scalar)
            } else {
                errors.push(SemanticError::WithShapeMismatch {
                    rule: rule.to_string(),
                    with: origin.to_string(),
                    detail: format!(
                        "regex capture `{g}` referenced via `{full_key}` is not in the captures of `{origin}`"
                    ),
                });
                None
            }
        }
        Hop::Field(f) => {
            // Legacy unqualified hop: check captures and child_ids (existing behaviour).
            if id_shape.captures.contains(f.as_str()) || id_shape.child_ids.contains_key(f.as_str())
            {
                // Resolve to child shape if it is a child id; otherwise scalar.
                if let Some(child_ref) = id_shape.child_ids.get(f.as_str()) {
                    Some(child_ref_to_shape(child_ref, rules))
                } else {
                    Some(Shape::Scalar)
                }
            } else {
                errors.push(SemanticError::WithShapeMismatch {
                    rule: rule.to_string(),
                    with: origin.to_string(),
                    detail: format!(
                        "field reference `{full_key}` is not in the derived id shape of `{origin}`"
                    ),
                });
                None
            }
        }
        Hop::Dir(id) | Hop::File(id) => {
            let requested_kind = match hop {
                Hop::Dir(_) => NodeKind::Dir,
                _ => NodeKind::File,
            };
            let requested_word = requested_kind.as_str();
            // Membership is decided by the child_ids of the current id shape; the actual
            // kind and the next shape are resolved from the global id_shapes map, since
            // child ids are globally unique and always carry an accurate IdShape there
            // (covers both Inline and RuleRef child refs uniformly).
            if !id_shape.child_ids.contains_key(id.as_str()) {
                errors.push(SemanticError::WithShapeMismatch {
                    rule: rule.to_string(),
                    with: origin.to_string(),
                    detail: format!(
                        "child id `{id}` referenced via `{full_key}` is not in the child ids of `{origin}`"
                    ),
                });
                return None;
            }
            match id_shapes.get(id.as_str()) {
                Some(child_id_shape) => {
                    if child_id_shape.kind != requested_kind {
                        let actual_word = child_id_shape.kind.as_str();
                        errors.push(SemanticError::NodeKindMismatch {
                            rule: rule.to_string(),
                            reference: full_key.to_string(),
                            expected: requested_word.to_string(),
                            actual: actual_word.to_string(),
                        });
                        None
                    } else {
                        Some(Shape::Id(child_id_shape.clone()))
                    }
                }
                // Not in the global map (theoretically rare): skip the kind check (lenient).
                None => Some(Shape::FreeForm),
            }
        }
        // choice/group/for/fetch hops navigate into the child record set keyed by `id`. The id
        // shape model (`NodeKind`) only distinguishes dir/file, so the kind comparison is skipped
        // here (lenient); only child-id membership is enforced when the shape is known.
        Hop::Choice(id) | Hop::Group(id) | Hop::For(id) | Hop::Fetch(id) => {
            if let Some(child_ref) = id_shape.child_ids.get(id.as_str()) {
                Some(child_ref_to_shape(child_ref, rules))
            } else {
                // The child set is produced by a non-dir/file entry whose id may not appear in the
                // statically derived `child_ids`; remain lenient (no false positive).
                Some(Shape::FreeForm)
            }
        }
    }
}

/// Converts a `ChildRef` to the `Shape` of the element (for multi-hop chaining).
fn child_ref_to_shape(child_ref: &ChildRef, rules: &IndexMap<RuleName, YamlRule>) -> Shape {
    match child_ref {
        ChildRef::Inline(id_shape) => Shape::Id(id_shape.clone()),
        ChildRef::RuleRef(rname) => match derive_rule_id_shape(rname, rules) {
            Some(id_shape) => Shape::Id(id_shape),
            None => Shape::FreeForm,
        },
    }
}

// ---------------------------------------------------------------------------
// For-source shape resolution
// ---------------------------------------------------------------------------

/// Resolves the element shape (and error-reporting origin label) produced by iterating
/// over a for-source expression.
///
/// Returns `(element_shape, origin_label)`:
/// - `Literal(_)` → `(Scalar, literal_text)`.
/// - `Expr(s)` where s is a single `${var}` → `(head_shape, head_origin)`.
/// - `Expr(s)` where s is a dotted `${head.hop1.hop2}` → follow the hops in `env`
///   and return the final shape and the head's origin label.
/// - Any other form → `(FreeForm, "")`.
fn resolve_for_source_shape(
    rule: &str,
    source: &YamlForSource,
    env: &ShapeEnv,
    rules: &IndexMap<RuleName, YamlRule>,
    id_shapes: &IndexMap<String, IdShape>,
    errors: &mut Vec<SemanticError>,
) -> (Shape, String) {
    match source {
        YamlForSource::Literal(_) => (Shape::Scalar, String::new()),
        YamlForSource::Expr(s) => {
            // Extract a single `${...}` reference that covers the entire expression.
            let Some(inner) = s.strip_prefix("${").and_then(|t| t.strip_suffix('}')) else {
                return (Shape::FreeForm, String::new());
            };
            if inner.contains('}') {
                return (Shape::FreeForm, String::new());
            }
            let parsed = parse_ref(inner);
            // `for n in ${value.names}`: a value list binds scalar string elements, so each
            // iteration element has scalar shape.
            if let RefHead::ValueNs { var, .. } = &parsed.head {
                return (Shape::Scalar, var.clone());
            }
            // Determine the env lookup key and effective hops based on namespace.
            // `use.<id>` and `for.<id>` are on the same path as `rule.<id>`: the wrapper is the
            // head and the tail hops navigate into it explicitly (no auto-unwrap).
            let (env_key, effective_hops): (&str, &[Hop]) = match &parsed.head {
                RefHead::Bare(name) => (name.as_str(), parsed.hops.as_slice()),
                RefHead::WithNs { param, tail } => (param.as_str(), tail.as_slice()),
                RefHead::RuleNs { rule_id, tail } => (rule_id.as_str(), tail.as_slice()),
                RefHead::UseNs { id, tail } => (id.as_str(), tail.as_slice()),
                RefHead::ForNs { id, tail } => (id.as_str(), tail.as_slice()),
                RefHead::FetchNs { id, tail } => (id.as_str(), tail.as_slice()),
                // Kind-qualified heads resolve on the same path as `for.<id>`: the head id is the
                // env key and the tail hops navigate into it. Strict kind matching is deferred.
                RefHead::DirNs { id, tail }
                | RefHead::FileNs { id, tail }
                | RefHead::GroupNs { id, tail }
                | RefHead::ChoiceNs { id, tail } => (id.as_str(), tail.as_slice()),
                // Handled above.
                RefHead::ValueNs { .. } => unreachable!("ValueNs handled above"),
            };
            let Some(head_entry) = env.get(env_key) else {
                // Unbound head: lenient (check_rule_var_scope handles the scope error).
                return (Shape::FreeForm, env_key.to_string());
            };
            let origin = head_entry.origin.clone();
            if effective_hops.is_empty() {
                // No tail hops: iterating an array binding yields its element shape; any other
                // binding iterates as itself (e.g. an id producing a record list of that shape).
                return (for_element_shape(&head_entry.shape), origin);
            }
            // Follow hops to find the final shape.
            let mut cur = head_entry.shape.clone();
            for hop in effective_hops {
                match step_shape(rule, inner, &origin, &cur, hop, rules, id_shapes, errors) {
                    Some(next) => cur = next,
                    None => return (Shape::FreeForm, origin),
                }
            }
            (for_element_shape(&cur), origin)
        }
    }
}

/// Returns the shape bound to each iteration variable when iterating `shape` in a `for`.
///
/// Iterating an `Array(T)` binds the element shape `T`. Every other shape iterates as itself (an
/// id producing a record list binds that record shape; a scalar/free-form binds unchanged).
fn for_element_shape(shape: &Shape) -> Shape {
    match shape {
        Shape::Array(elem) => (**elem).clone(),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Collects the pattern strings and input value texts from an entry.
fn pattern_and_input_texts(entry: &YamlEntry) -> Vec<String> {
    let mut texts = Vec::new();
    match &entry.kind {
        YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. } => {
            texts.push(pattern_str(pattern).to_string());
        }
        YamlEntryKind::Use { with_args, .. } => {
            for v in with_args.values() {
                texts.push(v.clone());
            }
        }
        YamlEntryKind::Value { value, .. } => {
            // The scalar / list elements of a value binding are templates; their `${...}`
            // references must be validated against the current scope.
            match value {
                ValueExpr::Scalar(s) => texts.push(s.clone()),
                ValueExpr::List(items) => texts.extend(items.iter().cloned()),
            }
        }
        _ => {}
    }
    texts
}

/// Returns a human-readable display string for a hop (used in error messages).
fn hop_display(hop: &Hop) -> String {
    match hop {
        Hop::Regex(g) => format!("regex.{g}"),
        Hop::Dir(id) => format!("dir.{id}"),
        Hop::File(id) => format!("file.{id}"),
        Hop::Choice(id) => format!("choice.{id}"),
        Hop::Group(id) => format!("group.{id}"),
        Hop::For(id) => format!("for.{id}"),
        Hop::Fetch(id) => format!("fetch.{id}"),
        Hop::Field(f) => f.clone(),
    }
}
