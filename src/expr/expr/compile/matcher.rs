use crate::expr::template_ref::strip_braces;
use crate::expr::{
    ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprSubtree, MatchArm, Quant,
};
use crate::yaml::{YamlEntry, YamlEntryKind, YamlForSource, YamlPattern};

use super::entry::to_expr_entry;
use super::pattern::to_expr_pattern;

pub fn build_matcher(entry: &YamlEntry, parent_path: Option<&str>) -> ExprMatcher {
    match &entry.kind {
        YamlEntryKind::Fetch { body } => {
            // Non-consuming observation entry. The fetch id is in entry.id (set by From<YamlEntryRepr>).
            ExprMatcher::Fetch {
                body: body
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let child_path = parent_path.map(|p| format!("{p}.fetch.of[{i}]"));
                        to_expr_entry(e, child_path.as_deref())
                    })
                    .collect(),
            }
        }

        YamlEntryKind::Match {
            scrutinee,
            body: arms,
        } => {
            let arms: Vec<MatchArm> = arms
                .iter()
                .enumerate()
                .map(|(i, arm_yaml)| {
                    let arm_entry = to_expr_entry(
                        arm_yaml,
                        parent_path
                            .map(|p| format!("{p}.match.rules[{i}]"))
                            .as_deref(),
                    );
                    // Validation guarantees each arm has `id: <tag>` and `rules: [...]` (Record-intro).
                    let tag = arm_entry
                        .id
                        .expect("match arm must have an id (guaranteed by validation)");
                    let subtree = match arm_entry.matcher {
                        ExprMatcher::Group { subtree } => subtree,
                        _ => panic!("match arm must be a Group (guaranteed by validation)"),
                    };
                    MatchArm { tag, subtree }
                })
                .collect();
            ExprMatcher::Match {
                scrutinee: strip_braces(scrutinee),
                arms,
            }
        }

        YamlEntryKind::For { var, source, body } => {
            let expr_source = match source {
                YamlForSource::Literal(v) => ExprForSource::Literal(v.clone()),
                YamlForSource::Expr(s) => ExprForSource::Expr(s.clone()),
            };
            ExprMatcher::For {
                var: var.clone(),
                source: expr_source,
                body: body
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let child_path = parent_path.map(|p| format!("{p}.for.rules[{i}]"));
                        to_expr_entry(e, child_path.as_deref())
                    })
                    .collect(),
            }
        }

        YamlEntryKind::Choice { min, max, body } => {
            // Choice (one_of/any_of/choice) entry.
            // Attaching optional to a choice is a syntax error at the YAML stage, so optional is not considered here.
            ExprMatcher::Choice {
                min: *min,
                max: *max,
                body: body
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let child_path = parent_path.map(|p| format!("{p}.group[{i}]"));
                        to_expr_entry(e, child_path.as_deref())
                    })
                    .collect(),
            }
        }

        YamlEntryKind::Use {
            rule: rule_name,
            with_args,
            ..
        } => {
            // A rule reference without its own dir/file (`- use: rule.X`) is a use entry (expansion at current position).
            // When `id:` is also present (`- use: rule.X / id: Y`), it is a use+id desugared to a Record
            // wrapping a bare Use so both surface forms share the single record-intro mechanism.
            // Assumption (post-validation): a bare rule does not coexist with rules (guaranteed by check_entry_combination).
            let use_entry = ExprMatcher::Use {
                rule: rule_name.clone(),
                with_args: with_args.clone(),
            };
            if entry.id.is_some() {
                // use+id → group record-intro wrapping a bare Use entry
                ExprMatcher::Group {
                    subtree: vec![ExprEntry {
                        id: None,
                        source_path: parent_path.map(|s| s.to_string()),
                        count: Quant::Default,
                        matcher: use_entry,
                    }],
                }
            } else {
                use_entry
            }
        }

        YamlEntryKind::Group { body, .. } => {
            // Anonymous group (record-intro): id + body, no dir/file/rule.
            // Assumption (post-validation): guaranteed by check_entry_combination.
            ExprMatcher::Group {
                subtree: body
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let child_path = parent_path.map(|p| format!("{p}.rules[{i}]"));
                        to_expr_entry(e, child_path.as_deref())
                    })
                    .collect(),
            }
        }

        YamlEntryKind::Value { var, value } => {
            // Value variable binding (`- id: x / value: ...`). Carried verbatim to the runtime,
            // where it is interpolated and bound into the `value` namespace.
            ExprMatcher::Value {
                var: var.clone(),
                value: value.clone(),
            }
        }

        YamlEntryKind::Dir { pattern, body, .. } => {
            // dir entry. Assumption (post-validation): dir entries do not coexist with rule.
            let (expr_pattern, subtree) =
                build_node_pattern_and_subtree(pattern, body, parent_path);
            ExprMatcher::Dir {
                pattern: expr_pattern,
                subtree,
            }
        }

        YamlEntryKind::File { pattern, body, .. } => {
            // file entry. Assumption (post-validation): file entries do not coexist with rule.
            let (expr_pattern, subtree) =
                build_node_pattern_and_subtree(pattern, body, parent_path);
            ExprMatcher::File {
                pattern: expr_pattern,
                subtree,
            }
        }
    }
}

/// Builds the pattern and subtree for a dir/file entry.
///
/// Strips the `/*` skip-contents marker from the end of the name before building the matcher.
fn build_node_pattern_and_subtree(
    pattern: &YamlPattern,
    body: &Option<Vec<YamlEntry>>,
    parent_path: Option<&str>,
) -> (ExprPattern, ExprSubtree) {
    let (matched_pattern, is_skip) = split_skip_marker(pattern);
    let expr_pattern = to_expr_pattern(&matched_pattern);
    let subtree = if let Some(inline) = body {
        ExprSubtree::Inline(
            inline
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let child_path = parent_path.map(|p| format!("{p}.rules[{i}]"));
                    to_expr_entry(e, child_path.as_deref())
                })
                .collect(),
        )
    } else if is_skip {
        // `/*` present: do not inspect contents (no descent).
        ExprSubtree::Leaf
    } else {
        // bare dir: contents must be empty (deny-by-default empty entry set).
        ExprSubtree::Inline(Vec::new())
    };
    (expr_pattern, subtree)
}

/// Strips the `/*` skip-contents marker from the end of the pattern name and returns it.
///
/// Returns the stripped name and `true` only when an exact name ends with `/*`.
/// Regex (`Spec`) or exact names without `/*` are returned as-is with `false`.
fn split_skip_marker(pattern: &YamlPattern) -> (YamlPattern, bool) {
    match pattern {
        YamlPattern::Exact(s) if s.ends_with("/*") => {
            (YamlPattern::Exact(s[..s.len() - 2].to_string()), true)
        }
        other => (other.clone(), false),
    }
}
