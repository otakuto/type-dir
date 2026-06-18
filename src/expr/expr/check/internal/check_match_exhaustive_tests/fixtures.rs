use crate::yaml::{
    EntryId, RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern, YamlRule,
};
use indexmap::IndexMap;

/// Builds an id-bearing alternative `- id: <id> / dir: <pattern>`.
fn alt(id: &str) -> YamlEntry {
    YamlEntry {
        id: Some(EntryId(id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact(format!("{id}-dir")),
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Builds an id-bearing `any_of` group (a Sum) whose alternatives are the given ids.
pub fn sum_group(group_id: &str, alt_ids: &[&str]) -> YamlEntry {
    let body = alt_ids.iter().map(|id| alt(id)).collect();
    YamlEntry {
        id: Some(EntryId(group_id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Choice {
            min: 1,
            max: None,
            body,
        },
    }
}

/// Builds a `match: ${<scrutinee>}` entry with the given arm ids (each arm is `- id: <tag> / rules: []`).
fn match_entry(scrutinee: &str, arm_ids: &[&str]) -> YamlEntry {
    let arms = arm_ids
        .iter()
        .map(|tag| YamlEntry {
            id: Some(EntryId(tag.to_string())),
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::Group {
                body: vec![],
                explicit_marker: true,
            },
        })
        .collect();
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Match {
            scrutinee: format!("${{{scrutinee}}}"),
            body: arms,
        },
    }
}

/// Builds a `for <var> in ${choice.<source>} / rules: [<match>]` entry wrapping a match.
pub fn for_match(var: &str, source: &str, scrutinee: &str, arm_ids: &[&str]) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName(var.to_string()),
            source: YamlForSource::Expr(format!("${{choice.{source}}}")),
            body: vec![match_entry(scrutinee, arm_ids)],
        },
    }
}

/// Builds a `for <var> in ${choice.<source>} / id: <for_id> / rules: [<one_of id: <choice_id>>]` entry.
///
/// The inner `one_of` has `id: <choice_id>` and alternatives with the given ids. This models the
/// pattern where an id-bearing for entry wraps an id-bearing choice, making `${for.<for_id>}` a
/// Sum whose constructors are the choice's alternative ids.
pub fn for_with_id_and_choice(
    var: &str,
    source: &str,
    for_id: &str,
    choice_id: &str,
    alt_ids: &[&str],
) -> YamlEntry {
    let inner_choice = sum_group(choice_id, alt_ids);
    YamlEntry {
        id: Some(EntryId(for_id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName(var.to_string()),
            source: YamlForSource::Expr(format!("${{choice.{source}}}")),
            body: vec![inner_choice],
        },
    }
}

/// Builds a `for <var> in ${for.<for_ns_id>} / rules: [match ${<scrutinee>}]` entry.
///
/// Uses the `${for.<for_ns_id>}` namespace source form. The inner match has arm ids `arm_ids`.
pub fn for_ns_match(var: &str, for_ns_id: &str, scrutinee: &str, arm_ids: &[&str]) -> YamlEntry {
    let arms = arm_ids
        .iter()
        .map(|tag| YamlEntry {
            id: Some(EntryId(tag.to_string())),
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::Group {
                body: vec![],
                explicit_marker: true,
            },
        })
        .collect();
    let match_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Match {
            scrutinee: format!("${{{scrutinee}}}"),
            body: arms,
        },
    };
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName(var.to_string()),
            source: YamlForSource::Expr(format!("${{for.{for_ns_id}}}")),
            body: vec![match_entry],
        },
    }
}

/// Builds a single-rule name→rule map whose `root` rule body is `entries`.
pub fn config_with(entries: Vec<YamlEntry>) -> IndexMap<RuleName, YamlRule> {
    let root_rule = YamlRule {
        rule: RuleName("root".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: entries,
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("root".to_string()), root_rule);
    rules
}
