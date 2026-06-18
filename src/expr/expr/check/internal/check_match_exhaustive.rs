#[cfg(test)]
#[path = "check_match_exhaustive_tests/tests.rs"]
mod tests;

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::expr::template_ref::{for_ns_id, kind_ns_id, strip_braces, tail_sum_id, value_ns_var};
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlForSource, YamlRule};

/// Checks `match` (Sum elimination) exhaustiveness (E024) and that the scrutinee is a Sum (E025).
///
/// A `match: ${c}` dispatches on the `tag` of a record bound to `c`. The only way a record acquires
/// a `tag` is by being collected from an id-bearing Group (one_of/any_of/choice with `id:`), whose
/// alternative ids are the possible tags (the Sum's constructors). Therefore the static rule is:
///
/// 1. `c` must be a `for` variable that iterates a bare `${items}` whose `items` is the id of an
///    id-bearing Group. Otherwise the scrutinee is not a Sum → `MatchOnNonSum` (E025).
/// 2. The set of arm ids must equal the set of alternative ids of that Group exactly. Tags with no
///    arm are `missing`; arm ids that are not alternatives are `extra` (dead arms). Any divergence
///    → `NonExhaustiveMatch` (E024).
///
/// Static identification uses a direct AST walk: while traversing each rule body, `for` bindings are
/// recorded as `var -> source id` (the bare `${items}` in `in:`). When a `match` is reached, the
/// scrutinee's source id is looked up, then the id-bearing Group with that id is found globally
/// (ids are globally unique) and its alternative ids are read off.
pub fn check_match_exhaustive(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    // Global map: id-bearing Group id -> its alternative ids (the Sum's constructors).
    let sum_tags = collect_sum_tags(rules);

    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        // for-binding map at the top level is empty (only with-params are in scope).
        let bindings: HashMap<String, String> = HashMap::new();
        walk(&rule_name.0, &rule.body, &bindings, &sum_tags, &mut errors);
    }
    errors
}

/// Collects every id-bearing Group's alternative ids into a global map keyed by the Group id.
///
/// An id-bearing Group is a `one_of`/`any_of`/`choice` entry that carries `entry.id`. Its
/// alternative ids are the `entry.id` of each alternative; alternatives without an id are skipped
/// (they cannot be a `tag`). Ids are globally unique, so a flat map is sufficient.
fn collect_sum_tags(rules: &IndexMap<RuleName, YamlRule>) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    for rule in rules.values() {
        collect_sum_tags_in(&rule.body, &mut map);
    }
    map
}

/// Recursively walks entries and records id-bearing Group alternative ids.
fn collect_sum_tags_in(entries: &[YamlEntry], map: &mut HashMap<String, Vec<String>>) {
    for entry in entries {
        match &entry.kind {
            YamlEntryKind::Choice { body, .. } => {
                if let Some(group_id) = &entry.id {
                    let tags: Vec<String> = body
                        .iter()
                        .filter_map(|alt| alt.id.as_ref().map(|id| id.0.clone()))
                        .collect();
                    map.insert(group_id.0.clone(), tags);
                }
                // Recurse into all child positions to register nested id-bearing Groups.
                collect_sum_tags_in(body, map);
            }
            YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } => {
                if let Some(inline) = body {
                    collect_sum_tags_in(inline, map);
                }
            }
            YamlEntryKind::Group { body, .. } => {
                collect_sum_tags_in(body, map);
            }
            YamlEntryKind::For { body, .. } => {
                // When this for entry carries an id, its collected records expose the tag of the
                // inner id-bearing Group as their own tag. Register `for_id -> alt_ids` so that
                // `${for.<for_id>}` can be resolved as a Sum by check_one_match.
                if let Some(for_id) = &entry.id {
                    let alt_ids = first_choice_alt_ids(body);
                    if !alt_ids.is_empty() {
                        map.insert(for_id.0.clone(), alt_ids);
                    }
                }
                collect_sum_tags_in(body, map);
            }
            YamlEntryKind::Match { body, .. } => {
                collect_sum_tags_in(body, map);
            }
            YamlEntryKind::Fetch { body } => {
                collect_sum_tags_in(body, map);
            }
            YamlEntryKind::Use { .. } => {}
            // a value binding introduces no Sum tags and owns no children
            YamlEntryKind::Value { .. } => {}
        }
    }
}

/// Returns the alternative ids of the first id-bearing Choice entry found in `entries`.
///
/// Used to determine what tags a `for` entry with an id will produce: the runtime collector lifts
/// the winning alternative's id as the record tag, so the for-id acts as a Sum whose constructors
/// are those alternative ids.
fn first_choice_alt_ids(entries: &[YamlEntry]) -> Vec<String> {
    for entry in entries {
        if let YamlEntryKind::Choice { body, .. } = &entry.kind
            && entry.id.is_some()
        {
            return body
                .iter()
                .filter_map(|alt| alt.id.as_ref().map(|id| id.0.clone()))
                .collect();
        }
    }
    vec![]
}

/// Walks a rule body, tracking `for` bindings (`var -> source id`), and validates each `match`.
///
/// `bindings` maps a `for` variable name to the bare source id it iterates (`for c in ${items}`
/// records `c -> "items"`). A `for` whose source is not a bare `${id}` records no binding (the
/// scrutinee would then resolve to no Sum → E025).
fn walk(
    rule: &str,
    entries: &[YamlEntry],
    bindings: &HashMap<String, String>,
    sum_tags: &HashMap<String, Vec<String>>,
    errors: &mut Vec<SemanticError>,
) {
    for entry in entries {
        match &entry.kind {
            YamlEntryKind::Match { scrutinee, body } => {
                // `scrutinee` is the raw `${...}` template; reduce it to the inner key and then to
                // the bare iteration-variable name (the scrutinee is written `${value.<var>}`).
                let scrutinee_key = strip_braces(scrutinee);
                let scrutinee_var = value_ns_var(&scrutinee_key);
                check_one_match(rule, scrutinee_var, body, bindings, sum_tags, errors);
                // Descend into arms.
                walk(rule, body, bindings, sum_tags, errors);
            }
            YamlEntryKind::Choice { body, .. } => {
                walk(rule, body, bindings, sum_tags, errors);
            }
            YamlEntryKind::Dir { body, .. } | YamlEntryKind::File { body, .. } => {
                if let Some(inline) = body {
                    walk(rule, inline, bindings, sum_tags, errors);
                }
            }
            YamlEntryKind::Group { body, .. } => {
                walk(rule, body, bindings, sum_tags, errors);
            }
            YamlEntryKind::For { var, source, body } => {
                let mut inner = bindings.clone();
                if let YamlForSource::Expr(s) = source {
                    if let Some(for_id) = for_ns_id(s) {
                        // `${for.<id>}` form: bind var -> "<id>" (the for-entry id acting as Sum).
                        inner.insert(var.0.clone(), for_id.to_string());
                    } else if let Some(id) = kind_ns_id(s) {
                        // `${choice.<id>}` / `${group.<id>}` / `${dir.<id>}` / `${file.<id>}` form:
                        // bind var -> "<id>" (kind-namespaced source acting as Sum).
                        inner.insert(var.0.clone(), id.to_string());
                    } else if let Some(id) = tail_sum_id(s) {
                        // Path source ending in `.choice.<id>` / `.group.<id>`
                        // (e.g. `${dir.components.choice.items}`): the trailing Sum id is the Sum.
                        inner.insert(var.0.clone(), id.to_string());
                    }
                }
                walk(rule, body, &inner, sum_tags, errors);
            }
            YamlEntryKind::Fetch { body } => {
                walk(rule, body, bindings, sum_tags, errors);
            }
            YamlEntryKind::Use { .. } => {}
            // a value binding contains no match and owns no children
            YamlEntryKind::Value { .. } => {}
        }
    }
}

/// Validates a single `match` against its scrutinee's Sum tags.
fn check_one_match(
    rule: &str,
    scrutinee: &str,
    arm_rules: &[YamlEntry],
    bindings: &HashMap<String, String>,
    sum_tags: &HashMap<String, Vec<String>>,
    errors: &mut Vec<SemanticError>,
) {
    // The scrutinee must be a `for` variable bound to a bare `${items}` source id.
    let Some(source_id) = bindings.get(scrutinee) else {
        errors.push(SemanticError::MatchOnNonSum {
            rule: rule.to_string(),
            scrutinee: scrutinee.to_string(),
        });
        return;
    };
    // That source id must be an id-bearing Group (a Sum).
    let Some(tags) = sum_tags.get(source_id) else {
        errors.push(SemanticError::MatchOnNonSum {
            rule: rule.to_string(),
            scrutinee: scrutinee.to_string(),
        });
        return;
    };

    // Arm ids (each arm is `- id: tag / rules: [...]`).
    let arm_ids: Vec<String> = arm_rules
        .iter()
        .filter_map(|arm| arm.id.as_ref().map(|id| id.0.clone()))
        .collect();

    let missing: Vec<String> = tags
        .iter()
        .filter(|t| !arm_ids.contains(t))
        .cloned()
        .collect();
    let extra: Vec<String> = arm_ids
        .iter()
        .filter(|a| !tags.contains(a))
        .cloned()
        .collect();

    if !missing.is_empty() || !extra.is_empty() {
        errors.push(SemanticError::NonExhaustiveMatch {
            rule: rule.to_string(),
            scrutinee: scrutinee.to_string(),
            missing,
            extra,
        });
    }
}
