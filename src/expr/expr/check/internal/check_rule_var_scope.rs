#[cfg(test)]
#[path = "check_rule_var_scope_tests/tests.rs"]
mod tests;

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::expr::template_ref::extract_refs;
use crate::expr::{Hop, RefHead, parse_ref};
use crate::yaml::{
    RuleName, ValueExpr, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern, YamlRule,
};

use super::pattern_util::{named_captures, pattern_str};

/// Traverses each rule's body and checks that `${...}` references are within the declared scope.
///
/// Under the `rule.`/`with.` namespace grammar, references are classified as:
/// - `${rule.<id>.<hops>}` (`RefHead::RuleNs`): `<id>` must be a self-owned id (splice+id alias).
///   `Hop::Field` in tail → `UnqualifiedReference` (E023); unbound `<id>` → `RuleUndeclaredRef` (E010).
/// - `${with.<param>.<hops>}` (`RefHead::WithNs`): `<param>` must be a declared with-param.
///   `Hop::Field` in tail → E023; unbound `<param>` → E010.
/// - `${<name>}` or `${<name>.<hops>}` (`RefHead::Bare`): rejected unconditionally as
///   `BareReference` (SM022). Every reference must name a namespace head; bare names (including
///   for-variables, ids, and captures) are no longer permitted — use `${dir.<id>}` / `${file.<id>}`
///   / `${value.<var>}` / `${with.<param>}` etc.
/// - `${value.<var>.<hops>}` (`RefHead::ValueNs`): `<var>` must be either a `value:` binding
///   introduced by an earlier sibling (sequential-let) or a `for` iteration variable (visible for
///   the whole for body). Unbound `<var>` → `RuleUndeclaredRef` (E010).
/// - for source (`for: {value: ${expr}}`): evaluated in the outer scope (iter var not yet bound).
///
/// Self-owned ids are enumerated by the (A') transparency rule (`collect_self_owned_ids`).
/// Entries directly under `roots` are not rules and are therefore excluded.
pub fn check_rule_var_scope(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        // with-params: only accessible via `with.` prefix (RefHead::WithNs).
        let with_params: HashSet<String> = rule.with_params.keys().map(|v| v.0.clone()).collect();
        // all_owned: all self-owned ids in the rule body (for distinguishing forward-reference vs. truly undeclared).
        let mut all_owned: HashSet<String> = HashSet::new();
        collect_self_owned_ids(&rule.body, &mut all_owned);
        // id -> declaring kind (rule-global), used to validate kind-namespaced references.
        let mut owned_kinds: HashMap<String, IdKind> = HashMap::new();
        collect_self_owned_kinds(&rule.body, &mut owned_kinds);
        // local_bound starts empty; check_entries adds ids sequentially as siblings are visited.
        let local_bound: HashSet<String> = HashSet::new();
        // value bindings: only accessible via `value.` prefix, in sequential-let order.
        let value_bound: HashSet<String> = HashSet::new();
        check_entries(
            &rule_name.to_string(),
            &rule.body,
            &local_bound,
            &with_params,
            &value_bound,
            &all_owned,
            &owned_kinds,
            &mut errors,
        );
    }
    errors
}

/// Collects self-owned ids from the rule body by the (A') transparency rule into `ids`.
///
/// Exported so that `check_with_shapes` can enumerate self-owned ids when building
/// the per-rule shape environment.
///
/// Rule (encapsulation): "id-less Own dir/file entries are **opaque** — their inner ids are hidden
/// (no longer bubble out; references into them require a path through an id-bearing ancestor).
/// Id-bearing Own entries **record that id and stop** (ids beyond it belong to that id's children
/// side). one_of/any_of/choice/group/for/match are transparent." Does not descend into splice
/// (bare rule reference) targets,
/// since validation is hermetic per rule and the ids of a splice target are not self-owned by
/// the current rule. The enforce-side `collect_visible_ids` is also splice-opaque, so both
/// share the same semantics for "self-owned id" (splice-target ids are supplied when the splice
/// is expanded, owned by that rule).
///
/// For splice+id entries (`- use: rule.X / id: Y`), the renamed id `Y` is self-owned by the current
/// rule (callers can reference `${Y}`), so it is recorded and iteration stops (the expanded ids
/// of rule X are hidden behind Y).
pub(crate) fn collect_self_owned_ids(entries: &[YamlEntry], ids: &mut HashSet<String>) {
    for entry in entries {
        // Id-bearing entries: record and stop (ids beyond belong to children side).
        // This handles both Own entries (dir/file with id) and splice+id entries (bare rule with id).
        if let Some(id) = &entry.id {
            ids.insert(id.0.clone());
            continue;
        }
        match &entry.kind {
            // group / for / match: transparent
            YamlEntryKind::Choice { body, .. } => {
                collect_self_owned_ids(body, ids);
            }
            YamlEntryKind::For { body, .. } => {
                collect_self_owned_ids(body, ids);
            }
            YamlEntryKind::Match { body, .. } => {
                collect_self_owned_ids(body, ids);
            }
            // fetch: its id (now entry.id) is already handled above via `if let Some(id)`.
            // (Fetch always has entry.id set by From<YamlEntryRepr>.)
            YamlEntryKind::Fetch { .. } => {}
            // Id-less dir/file: opaque (encapsulation). Inner ids are hidden from the outer scope;
            // references into them require a path through an id-bearing ancestor.
            YamlEntryKind::Dir { .. } | YamlEntryKind::File { .. } => {}
            // Use without id: no id to collect (use-opaque).
            YamlEntryKind::Use { .. } => {}
            // Group without id: transparent to subtree.
            YamlEntryKind::Group { body, .. } => {
                collect_self_owned_ids(body, ids);
            }
            // Value binding: its name lives in the `value` namespace, not the self-owned id set,
            // and it owns no children (leaf).
            YamlEntryKind::Value { .. } => {}
        }
    }
}

/// The declaring kind of a self-owned id, used to validate kind-namespaced references
/// (`${dir.<id>}` must target a dir id, `${file.<id>}` a file id, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdKind {
    Dir,
    File,
    Group,
    Choice,
    /// `for`/`fetch`/`use`+id and any other id producer not addressed by `dir.`/`file.`/`group.`/
    /// `choice.`. The kind check does not constrain these (they use their own namespaces).
    Other,
}

impl IdKind {
    /// Returns the namespace keyword (`"dir"`/`"file"`/`"group"`/`"choice"`) this kind expects,
    /// or `None` for `Other` (no kind constraint).
    fn ns_keyword(self) -> Option<&'static str> {
        match self {
            IdKind::Dir => Some("dir"),
            IdKind::File => Some("file"),
            IdKind::Group => Some("group"),
            IdKind::Choice => Some("choice"),
            IdKind::Other => None,
        }
    }
}

/// Collects the declaring kind of every self-owned id in the rule body (mirrors the opaque/stop
/// traversal of `collect_self_owned_ids`). The map is rule-global since each id is declared once.
pub(crate) fn collect_self_owned_kinds(entries: &[YamlEntry], kinds: &mut HashMap<String, IdKind>) {
    for entry in entries {
        if let Some(id) = &entry.id {
            // Record-intro group (`- id: X / ::`) is a Group; choice/dir/file follow the matcher.
            let kind = match &entry.kind {
                YamlEntryKind::Dir { .. } => IdKind::Dir,
                YamlEntryKind::File { .. } => IdKind::File,
                YamlEntryKind::Choice { .. } => IdKind::Choice,
                YamlEntryKind::Group { .. } => IdKind::Group,
                // for/fetch/use+id: use their own namespaces; no dir/file/group/choice constraint.
                _ => IdKind::Other,
            };
            kinds.insert(id.0.clone(), kind);
            continue;
        }
        match &entry.kind {
            YamlEntryKind::Choice { body, .. } => {
                collect_self_owned_kinds(body, kinds);
            }
            YamlEntryKind::For { body, .. } => collect_self_owned_kinds(body, kinds),
            YamlEntryKind::Match { body, .. } => collect_self_owned_kinds(body, kinds),
            YamlEntryKind::Fetch { .. } => {}
            // Id-less dir/file: opaque (encapsulation) — do not descend.
            YamlEntryKind::Dir { .. } | YamlEntryKind::File { .. } => {}
            YamlEntryKind::Use { .. } => {}
            YamlEntryKind::Group { body, .. } => collect_self_owned_kinds(body, kinds),
            YamlEntryKind::Value { .. } => {}
        }
    }
}

/// Traverses entries and checks the references in each entry.
///
/// `local_bound` is the set of locally-bound names reachable at this position (self-owned ids +
/// ancestor captures). With-params are in `with_params` and are only valid when accessed via
/// `RefHead::WithNs` (`${with.<param>}`). `value_bound` is the set of names accessed via
/// `RefHead::ValueNs` (`${value.<var>}`): `value:` bindings and `for` iteration variables.
///
/// Value bindings follow sequential-let semantics within a `rules` block: a `value:` entry adds its
/// name to a running set so that subsequent sibling entries (and their descendants) can reference it,
/// but earlier siblings cannot.
#[allow(clippy::too_many_arguments)]
fn check_entries(
    rule: &str,
    entries: &[YamlEntry],
    local_bound: &HashSet<String>,
    with_params: &HashSet<String>,
    value_bound: &HashSet<String>,
    all_owned: &HashSet<String>,
    owned_kinds: &HashMap<String, IdKind>,
    errors: &mut Vec<SemanticError>,
) {
    // Running value-binding set: accumulates `value:` bindings as siblings are visited (sequential-let).
    let mut value_bound = value_bound.clone();
    // Running local set: accumulates self-owned ids as siblings are visited (sequential-let for ids).
    let mut running_local = local_bound.clone();

    for entry in entries {
        // Collect the pattern(s) of this entry (Node kind only).
        let patterns: Vec<&YamlPattern> = match &entry.kind {
            YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. } => {
                vec![pattern]
            }
            _ => vec![],
        };

        // Check references in the entry's own dir/file pattern
        for pattern in &patterns {
            for reference in extract_refs(pattern_str(pattern)) {
                check_ref(
                    rule,
                    &reference,
                    &running_local,
                    with_params,
                    &value_bound,
                    all_owned,
                    owned_kinds,
                    errors,
                );
            }
        }

        // Check references in with-args values (Use kind only)
        if let YamlEntryKind::Use { with_args, .. } = &entry.kind {
            for value in with_args.values() {
                for reference in extract_refs(value) {
                    check_ref(
                        rule,
                        &reference,
                        &running_local,
                        with_params,
                        &value_bound,
                        all_owned,
                        owned_kinds,
                        errors,
                    );
                }
            }
        }

        // Check references in a value binding's templates, then add its name to the running set.
        if let YamlEntryKind::Value { var, value } = &entry.kind {
            let texts: Vec<&str> = match value {
                ValueExpr::Scalar(s) => vec![s.as_str()],
                ValueExpr::List(items) => items.iter().map(String::as_str).collect(),
            };
            for text in texts {
                for reference in extract_refs(text) {
                    check_ref(
                        rule,
                        &reference,
                        &running_local,
                        with_params,
                        &value_bound,
                        all_owned,
                        owned_kinds,
                        errors,
                    );
                }
            }
            // Sequential-let: the binding becomes visible to later siblings and their descendants.
            value_bound.insert(var.0.clone());
        }

        // Scope for descendants: add own captures to running_local
        let mut child_local = running_local.clone();
        for pattern in &patterns {
            for capture in named_captures(pattern) {
                child_local.insert(capture);
            }
        }

        match &entry.kind {
            YamlEntryKind::Choice { body, .. } => {
                check_entries(
                    rule,
                    body,
                    &child_local,
                    with_params,
                    &value_bound,
                    all_owned,
                    owned_kinds,
                    errors,
                );
            }
            YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } => {
                if let Some(inline) = body {
                    check_entries(
                        rule,
                        inline,
                        &child_local,
                        with_params,
                        &value_bound,
                        all_owned,
                        owned_kinds,
                        errors,
                    );
                }
            }
            YamlEntryKind::Group { body, .. } => {
                check_entries(
                    rule,
                    body,
                    &child_local,
                    with_params,
                    &value_bound,
                    all_owned,
                    owned_kinds,
                    errors,
                );
            }
            YamlEntryKind::For { var, source, body } => {
                // for source is evaluated in the outer scope (iter var not yet bound)
                if let YamlForSource::Expr(s) = source {
                    for reference in extract_refs(s) {
                        check_ref(
                            rule,
                            &reference,
                            &child_local,
                            with_params,
                            &value_bound,
                            all_owned,
                            owned_kinds,
                            errors,
                        );
                    }
                }
                // The iteration variable is bound in the `value` namespace and is referenced from
                // the body as `${value.<itervar>}` (`RefHead::ValueNs`). Unlike `value:` bindings
                // (sequential-let, visible only to later siblings), it is visible for the whole for
                // body, so add it to a body-scoped clone of `value_bound`.
                let mut for_value_bound = value_bound.clone();
                for_value_bound.insert(var.0.clone());
                check_entries(
                    rule,
                    body,
                    &child_local,
                    with_params,
                    &for_value_bound,
                    all_owned,
                    owned_kinds,
                    errors,
                );
            }
            YamlEntryKind::Match { scrutinee, body } => {
                // match entry: the scrutinee `${c}` must reference a bound variable, then recursively
                // check arm_rules with the same bound scope (arms check their own subtrees).
                for reference in extract_refs(scrutinee) {
                    check_ref(
                        rule,
                        &reference,
                        &child_local,
                        with_params,
                        &value_bound,
                        all_owned,
                        owned_kinds,
                        errors,
                    );
                }
                check_entries(
                    rule,
                    body,
                    &child_local,
                    with_params,
                    &value_bound,
                    all_owned,
                    owned_kinds,
                    errors,
                );
            }
            YamlEntryKind::Fetch { body } => {
                // fetch entry: the alts are dir/file patterns scoped to the current level.
                // The fetch id is a new binding (already added to child_local via collect_self_owned_ids).
                check_entries(
                    rule,
                    body,
                    &child_local,
                    with_params,
                    &value_bound,
                    all_owned,
                    owned_kinds,
                    errors,
                );
            }
            YamlEntryKind::Use { .. } => {
                // Use: input values already checked above; no children to recurse into.
            }
            YamlEntryKind::Value { .. } => {
                // Value binding: templates already checked above; no children to recurse into.
            }
        }

        // Sequential-let for ids: once an entry is processed, its contributed ids become visible to subsequent siblings.
        let mut entry_ids: HashSet<String> = HashSet::new();
        collect_self_owned_ids(std::slice::from_ref(entry), &mut entry_ids);
        running_local.extend(entry_ids);
    }
}

/// Checks a single reference under the `rule.`/`with.`/`use.` namespace grammar.
///
/// Head dispatch:
/// - `RefHead::RuleNs { rule_id, tail }`: value-position use of `rule.` is an error (SM020).
///   `rule.` is valid only in type positions (with: declarations, `- use: rule.X` invoke targets).
///   Use `${use.<id>}` to reference a splice instance value.
/// - `RefHead::WithNs { param, tail }`: `param` must be in `with_params`.
///   `Hop::Field` in tail → E023; unbound → E010.
/// - `RefHead::ValueNs { var, tail }`: `var` must be in `value_bound` (a `value:` binding visible
///   under sequential-let). `Hop::Field` in tail → E023; unbound `var` → E010.
/// - `RefHead::UseNs { id, tail }`: `id` must be in `local_bound` (a self-owned splice id).
///   `Hop::Field` in tail → E023; unbound `id` → E010.
/// - `RefHead::ForNs { id, tail }` / `RefHead::FetchNs { id, tail }`: `id` must be a self-owned id
///   (a `for` / `fetch` entry in scope that carries `id: <id>`). `Hop::Field` in tail → E023;
///   forward-reference → SM (ForwardReference); otherwise unbound → E010.
/// - `RefHead::DirNs` / `RefHead::FileNs` / `RefHead::GroupNs` / `RefHead::ChoiceNs` `{ id, tail }`:
///   `id` must be a self-owned id in scope (kind-qualified reference). Resolved like `ForNs`:
///   `Hop::Field` in tail → E023; forward-reference → ForwardReference; otherwise unbound → E010.
///   Strict kind matching against the entry's actual kind is deferred to a later change.
///   When the head id is bound and its declaring kind is known, a kind mismatch (`${dir.X}` for a
///   file id, etc.) is reported as `NodeKindMismatch`.
/// - `RefHead::Bare(name)`: always rejected as `BareReference` (references must be namespaced).
#[allow(clippy::too_many_arguments)]
fn check_ref(
    rule: &str,
    reference: &str,
    local_bound: &HashSet<String>,
    with_params: &HashSet<String>,
    value_bound: &HashSet<String>,
    all_owned: &HashSet<String>,
    owned_kinds: &HashMap<String, IdKind>,
    errors: &mut Vec<SemanticError>,
) {
    let parsed = parse_ref(reference);
    match &parsed.head {
        RefHead::ValueNs { var, tail } => {
            if tail.iter().any(|h| matches!(h, Hop::Field(_))) {
                errors.push(SemanticError::UnqualifiedReference {
                    reference: reference.to_string(),
                    context: rule.to_string(),
                });
            } else if !value_bound.contains(var.as_str()) {
                errors.push(SemanticError::RuleUndeclaredRef {
                    rule: rule.to_string(),
                    reference: reference.to_string(),
                });
            }
        }
        RefHead::RuleNs { rule_id, .. } => {
            // `${rule.X}` in a value position is always an error: `rule.` is a type namespace,
            // not a value namespace. Use `${use.<id>}` to reference a splice instance value.
            errors.push(SemanticError::RuleNsInValuePosition {
                rule: rule.to_string(),
                rule_id: rule_id.clone(),
            });
        }
        RefHead::WithNs { param, tail } => {
            if tail.iter().any(|h| matches!(h, Hop::Field(_))) {
                errors.push(SemanticError::UnqualifiedReference {
                    reference: reference.to_string(),
                    context: rule.to_string(),
                });
            } else if !with_params.contains(param.as_str()) {
                errors.push(SemanticError::RuleUndeclaredRef {
                    rule: rule.to_string(),
                    reference: reference.to_string(),
                });
            }
        }
        RefHead::UseNs { id, tail } => {
            if tail.iter().any(|h| matches!(h, Hop::Field(_))) {
                errors.push(SemanticError::UnqualifiedReference {
                    reference: reference.to_string(),
                    context: rule.to_string(),
                });
            } else if !local_bound.contains(id.as_str()) {
                if all_owned.contains(id.as_str()) {
                    errors.push(SemanticError::ForwardReference {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                        id: id.clone(),
                    });
                } else {
                    errors.push(SemanticError::RuleUndeclaredRef {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                    });
                }
            }
        }
        RefHead::ForNs { id, tail } => {
            // `${for.X}` references a for-entry with id X. X must be a self-owned id (i.e. a for
            // entry in scope that has `id: X`). The tail hops navigate into the wrapped record set.
            if tail.iter().any(|h| matches!(h, Hop::Field(_))) {
                errors.push(SemanticError::UnqualifiedReference {
                    reference: reference.to_string(),
                    context: rule.to_string(),
                });
            } else if !local_bound.contains(id.as_str()) {
                if all_owned.contains(id.as_str()) {
                    errors.push(SemanticError::ForwardReference {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                        id: id.clone(),
                    });
                } else {
                    errors.push(SemanticError::RuleUndeclaredRef {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                    });
                }
            }
        }
        RefHead::FetchNs { id, tail } => {
            // `${fetch.X}` references a fetch-entry with id X. X must be a self-owned id (i.e. a
            // fetch entry in scope that has `id: X`; `collect_self_owned_ids` records fetch ids via
            // `entry.id`). The tail hops navigate into the collected record set. Resolved on the
            // same path as `RefHead::ForNs`.
            if tail.iter().any(|h| matches!(h, Hop::Field(_))) {
                errors.push(SemanticError::UnqualifiedReference {
                    reference: reference.to_string(),
                    context: rule.to_string(),
                });
            } else if !local_bound.contains(id.as_str()) {
                if all_owned.contains(id.as_str()) {
                    errors.push(SemanticError::ForwardReference {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                        id: id.clone(),
                    });
                } else {
                    errors.push(SemanticError::RuleUndeclaredRef {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                    });
                }
            }
        }
        RefHead::DirNs { id, tail }
        | RefHead::FileNs { id, tail }
        | RefHead::GroupNs { id, tail }
        | RefHead::ChoiceNs { id, tail } => {
            // `${dir.X}` / `${file.X}` / `${group.X}` / `${choice.X}` kind-qualify a reference to a
            // self-owned id X. X must be a self-owned id in scope, and its declaring kind must match
            // the namespace keyword (when known). The tail hops navigate into the resolved record
            // set (structural validation of the tail is left to `check_with_shapes`).
            // The head keyword for the kind check (the parser guarantees exactly one of these heads).
            let head_keyword = match &parsed.head {
                RefHead::DirNs { .. } => "dir",
                RefHead::FileNs { .. } => "file",
                RefHead::GroupNs { .. } => "group",
                RefHead::ChoiceNs { .. } => "choice",
                _ => unreachable!("matched DirNs/FileNs/GroupNs/ChoiceNs"),
            };
            if tail.iter().any(|h| matches!(h, Hop::Field(_))) {
                errors.push(SemanticError::UnqualifiedReference {
                    reference: reference.to_string(),
                    context: rule.to_string(),
                });
            } else if !local_bound.contains(id.as_str()) {
                if all_owned.contains(id.as_str()) {
                    errors.push(SemanticError::ForwardReference {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                        id: id.clone(),
                    });
                } else {
                    errors.push(SemanticError::RuleUndeclaredRef {
                        rule: rule.to_string(),
                        reference: reference.to_string(),
                    });
                }
            } else if let Some(kind) = owned_kinds.get(id.as_str())
                && let Some(expected) = kind.ns_keyword()
                && expected != head_keyword
            {
                // The id is bound but its declaring kind disagrees with the namespace keyword.
                errors.push(SemanticError::NodeKindMismatch {
                    rule: rule.to_string(),
                    reference: reference.to_string(),
                    expected: head_keyword.to_string(),
                    actual: expected.to_string(),
                });
            }
        }
        RefHead::Bare(_name) => {
            // Bare (un-namespaced) references are no longer permitted: every reference must name a
            // namespace head. Reject unconditionally so that callers migrate to `${dir.<id>}` /
            // `${file.<id>}` / `${value.<var>}` / etc.
            errors.push(SemanticError::BareReference {
                rule: rule.to_string(),
                reference: reference.to_string(),
            });
        }
    }
}
