#[cfg(test)]
#[path = "check_entry_combination_tests/tests.rs"]
mod tests;

use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};

/// Recursively walks all rule entries and validates entry field combinations.
///
/// The valid entry forms are:
/// - dir/file entry: has dir or file. Contents go under `rules:` (inline). `use:` cannot coexist.
/// - splice entry: `use: rule.<name>` only (`with:` and `id:` allowed). dir/file/rules cannot coexist.
/// - anonymous group (record-intro): has `rules:` but no dir/file/use. Count keys are not allowed.
/// - group(one_of/any_of) / for entry: has its own dedicated fields.
///
/// Checks performed:
/// - `use:` coexisting with dir/file entry → `DirFileWithRule` (should be written as `rules: [use: rule.X]`)
/// - `rules:` coexisting with splice (`use:`) entry → `SpliceWithSubtree`
/// - `id:` on a splice entry is allowed (desugared to record-intro at compile time)
/// - anonymous group: count keys (optional/min/max/count) are not allowed
/// - entry with neither dir/file nor use nor group nor for nor rules → `DirFileWithRule` (no name descriptor)
/// - 4 count keys (optional/min/max/count) on group/for entry → `InvalidPattern`
/// - min/max/count on splice entry → `InvalidPattern` (only optional is allowed)
/// - XOR violation of `count` with {`optional`/`min`/`max`} → `InvalidPattern`
/// - XOR violation of `optional` with `min` → `InvalidPattern`
/// - `min > max` → `InvalidPattern`
/// - Exact pattern + effective min>=2 / max>=2 / count>=2 → `InvalidPattern`
/// - `optional: true` or `min: 0` on an alternative (entry inside a group) → `InvalidPattern`
pub fn check_entry_combination(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        for (index, entry) in rule.body.iter().enumerate() {
            let context = format!("rules.{}.rules[{}]", rule_name, index);
            check_entry_inner(entry, &context, false, &mut errors);
        }
    }
    errors
}

fn check_entry_inner(
    entry: &YamlEntry,
    context: &str,
    in_alternative: bool,
    errors: &mut Vec<SemanticError>,
) {
    match &entry.kind {
        YamlEntryKind::Choice { min, max, body } => {
            // group entry: the 4 count keys (optional/min/max/count) are not allowed
            if has_any_count_key(entry) {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: "optional/min/max/count can only be placed on dir/file/splice entries (not on group)"
                        .to_string(),
                });
            }
            // Validate choice cardinality (when max is Some(m), min > m is invalid; max: 0 is legal = forbidden)
            if let Some(max_val) = max
                && min > max_val
            {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: format!("choice min exceeds max (min={min}, max={max_val})"),
                });
            }
            for (index, alt) in body.iter().enumerate() {
                let alt_context = format!("{}.group[{}]", context, index);
                check_entry_inner(alt, &alt_context, true, errors);
            }
        }

        YamlEntryKind::For { body, .. } => {
            // for entry: the 4 count keys (optional/min/max/count) are not allowed
            if has_any_count_key(entry) {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: "optional/min/max/count can only be placed on dir/file/splice entries (not on for)"
                        .to_string(),
                });
            }
            for (index, child) in body.iter().enumerate() {
                let child_context = format!("{}.for.rules[{}]", context, index);
                check_entry_inner(child, &child_context, false, errors);
            }
        }

        YamlEntryKind::Match { body, .. } => {
            // match entry: count keys are not allowed; arm rules are recursively validated
            if has_any_count_key(entry) {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: "optional/min/max/count can only be placed on dir/file/splice entries (not on match)"
                        .to_string(),
                });
            }
            for (index, arm) in body.iter().enumerate() {
                let arm_context = format!("{}.match.rules[{}]", context, index);
                check_entry_inner(arm, &arm_context, false, errors);
            }
        }

        YamlEntryKind::Fetch { body } => {
            // fetch entry: count keys are not allowed; alts must be dir/file only (no nested group/for/rule).
            if has_any_count_key(entry) {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: "optional/min/max/count can only be placed on dir/file/splice entries (not on fetch)"
                        .to_string(),
                });
            }
            for (index, alt) in body.iter().enumerate() {
                let alt_context = format!("{}.fetch.of[{}]", context, index);
                check_entry_inner(alt, &alt_context, false, errors);
            }
        }

        YamlEntryKind::Dir {
            pattern,
            body,
            colocated_use_ref,
        }
        | YamlEntryKind::File {
            pattern,
            body,
            colocated_use_ref,
        } => {
            let is_dir = matches!(entry.kind, YamlEntryKind::Dir { .. });
            // dir/file entry colocated with use: → DirFileWithRule error.
            if colocated_use_ref.is_some() {
                errors.push(SemanticError::DirFileWithRule {
                    context: context.to_string(),
                });
                return;
            }
            // Validate the 4 count keys (only allowed on dir/file entries)
            check_count_keys_node(entry, pattern, context, errors);
            // optional:true / min:0 on an alternative is a hollowing error
            if in_alternative {
                check_alternative_count(entry, context, errors);
            }
            // The `/*` (skip-contents) marker is only valid on dir entries that have no inline `::` block.
            if let YamlPattern::Exact(s) = pattern
                && s.ends_with("/*")
            {
                if !is_dir {
                    errors.push(SemanticError::InvalidPattern {
                        context: context.to_string(),
                        reason: "`/*` (skip contents) is only valid on a dir entry, not a file"
                            .to_string(),
                    });
                } else if body.is_some() {
                    errors.push(SemanticError::InvalidPattern {
                        context: context.to_string(),
                        reason: "`/*` (skip contents) cannot coexist with an inline `::` block"
                            .to_string(),
                    });
                }
            }
            // Recursively check inline body
            if let Some(inline) = body {
                for (index, child) in inline.iter().enumerate() {
                    let child_context = format!("{}.rules[{}]", context, index);
                    check_entry_inner(child, &child_context, false, errors);
                }
            }
        }

        YamlEntryKind::Use {
            colocated_rules, ..
        } => {
            // use entry: colocated rules coexistence is forbidden; id is allowed (desugared to record-intro)
            if colocated_rules.is_some() {
                errors.push(SemanticError::SpliceWithSubtree {
                    context: context.to_string(),
                });
            }
            // A use (bare rule) entry may only have optional.
            if entry.min.is_some() || entry.max.is_some() || entry.count.is_some() {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: "min/max/count can only be placed on dir/file entries (splice entries only accept optional)"
                        .to_string(),
                });
            }
            // optional:true / min:0 on an alternative is a hollowing error
            if in_alternative {
                check_alternative_count(entry, context, errors);
            }
        }

        YamlEntryKind::Value { .. } => {
            // Value binding (`- id: x / value: ...`). The repr (`ValueRepr` with
            // `deny_unknown_fields` and a required `id`) already guarantees `value:` cannot coexist
            // with dir/file/rule/group/for/match/optional/min/max/count keys, and that `id` is
            // present, so there is no additional combination to validate here.
        }

        YamlEntryKind::Group {
            body,
            explicit_marker,
        } => {
            // An implicit group — `::` (`body:`) with no dir/file/use and no explicit `group:`
            // marker — is no longer valid. `From<YamlEntryRepr>` produces such entries with
            // `explicit_marker == false`. Reject with `ImplicitGroup`, directing the author to add
            // `group:`. (The `explicit_marker == false` + empty body + no id case is the "no name
            // descriptor" sentinel — an entry with no matcher at all — reported as `DirFileWithRule`.)
            if !explicit_marker {
                if body.is_empty() && entry.id.is_none() {
                    errors.push(SemanticError::DirFileWithRule {
                        context: context.to_string(),
                    });
                } else {
                    errors.push(SemanticError::ImplicitGroup {
                        context: context.to_string(),
                    });
                }
                return;
            }
            // Explicit `group:` record-intro: non-consuming, multiplicity exactly 1.
            // count/optional keys are not allowed (same as group/for).
            if has_any_count_key(entry) {
                errors.push(SemanticError::InvalidPattern {
                    context: context.to_string(),
                    reason: "optional/min/max/count can only be placed on dir/file/splice entries (not on a group)"
                        .to_string(),
                });
            }
            // Recurse into inline body.
            for (index, child) in body.iter().enumerate() {
                let child_context = format!("{}.rules[{}]", context, index);
                check_entry_inner(child, &child_context, false, errors);
            }
        }
    }
}

/// Returns true if the entry has any of the 4 count keys (optional/min/max/count).
fn has_any_count_key(entry: &YamlEntry) -> bool {
    entry.optional.is_some() || entry.min.is_some() || entry.max.is_some() || entry.count.is_some()
}

/// Checks that an alternative entry does not have a setting that yields effective min=0.
///
/// `optional: true` or `min: 0` hollows out the disjunction (the choice is always satisfied,
/// making the selection meaningless) and is therefore an error.
/// `min: 1` or higher, `max` alone, and `count` are legal (used in satisfiability check `[c_a in Q_a]`).
fn check_alternative_count(entry: &YamlEntry, context: &str, errors: &mut Vec<SemanticError>) {
    if entry.optional == Some(true) {
        errors.push(SemanticError::InvalidPattern {
            context: context.to_string(),
            reason:
                "optional: true on an alternative hollows out the disjunction and is not allowed"
                    .to_string(),
        });
    }
    if entry.min == Some(0) {
        errors.push(SemanticError::InvalidPattern {
            context: context.to_string(),
            reason: "min: 0 on an alternative hollows out the disjunction and is not allowed"
                .to_string(),
        });
    }
}

/// Validates the 4 count keys (optional/min/max/count) of a Node (dir/file) entry.
///
/// XOR rules:
/// - `count` coexisting with {`optional`/`min`/`max`} → InvalidPattern
/// - `optional` coexisting with `min` → InvalidPattern (checked by key presence, not value)
///
/// `min` + `max` coexistence is legal.
///
/// Value constraints:
/// - `min > max` → InvalidPattern
/// - Exact pattern + effective min>=2 / max>=2 / count>=2 → InvalidPattern (name uniqueness violation)
fn check_count_keys_node(
    entry: &YamlEntry,
    pattern: &YamlPattern,
    context: &str,
    errors: &mut Vec<SemanticError>,
) {
    let has_optional = entry.optional.is_some();
    let has_min = entry.min.is_some();
    let has_max = entry.max.is_some();
    let has_count = entry.count.is_some();

    // All 4 keys absent — nothing to check
    if !has_optional && !has_min && !has_max && !has_count {
        return;
    }

    // XOR: `count` cannot coexist with {`optional`/`min`/`max`}
    if has_count && (has_optional || has_min || has_max) {
        errors.push(SemanticError::InvalidPattern {
            context: context.to_string(),
            reason: "count cannot coexist with optional/min/max".to_string(),
        });
        return; // Early return because subsequent checks assume count is absent
    }

    // XOR: `optional` cannot coexist with `min` (checked by key presence, not value)
    if has_optional && has_min {
        errors.push(SemanticError::InvalidPattern {
            context: context.to_string(),
            reason: "optional and min cannot coexist (optional is syntactic sugar for min)"
                .to_string(),
        });
        return;
    }

    let is_exact = YamlPattern::is_exact(pattern);

    // count scalar: validate {n, n} consistency
    if let Some(n) = entry.count {
        if is_exact && n >= 2 {
            errors.push(SemanticError::InvalidPattern {
                context: context.to_string(),
                reason: format!(
                    "Exact pattern count cannot be 2 or more due to name uniqueness (count={n})"
                ),
            });
        }
        return;
    }

    // min/max: validate value consistency
    if let (Some(min), Some(max)) = (entry.min, entry.max)
        && min > max
    {
        errors.push(SemanticError::InvalidPattern {
            context: context.to_string(),
            reason: format!("count min exceeds max (min={min}, max={max})"),
        });
    }

    // Exact patterns are constrained to c_e in {0, 1} due to name uniqueness.
    // max Some(m >= 2) is meaningless and min >= 2 is unreachable, so both are errors.
    if is_exact {
        if let Some(max) = entry.max
            && max >= 2
        {
            errors.push(SemanticError::InvalidPattern {
                context: context.to_string(),
                reason: "Exact pattern count max cannot exceed 1".to_string(),
            });
        }
        if entry.min.is_some_and(|min| min >= 2) {
            errors.push(SemanticError::InvalidPattern {
                context: context.to_string(),
                reason: "Exact pattern count min cannot exceed 1".to_string(),
            });
        }
    }
}
