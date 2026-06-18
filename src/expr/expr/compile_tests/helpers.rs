use crate::yaml::{RuleName, YamlConfig, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};
use indexmap::IndexMap;

/// Creates a bare rule splice entry (`- use: rule.X`).
pub fn make_splice_entry(rule_name: &str) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName(rule_name.to_string()),
            with_args: IndexMap::new(),
            colocated_rules: None,
        },
    }
}

/// Creates a rule whose body is a single `dir:` Own entry (with the given entries as contents).
/// Equivalent representation of the legacy name-owning rule (node moved to the entry side).
pub fn make_dir_rule(dir_name: &str, entries: Vec<YamlEntry>) -> YamlRule {
    let dir_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact(dir_name.to_string()),
            body: if entries.is_empty() {
                None
            } else {
                Some(entries)
            },
            colocated_use_ref: None,
        },
    };
    YamlRule {
        rule: RuleName("".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![dir_entry],
    }
}

/// Creates a minimal YamlConfig containing a root rule that splices `child_name` and a child rule.
pub fn make_minimal_yaml(
    child_name: &str,
    mut child_rule: YamlRule,
    extra_rules: IndexMap<RuleName, YamlRule>,
) -> YamlConfig {
    // Ensure the child_rule carries the correct rule name so compile() maps it correctly.
    child_rule.rule = RuleName(child_name.to_string());
    let mut rules = extra_rules;
    rules.insert(RuleName(child_name.to_string()), child_rule);
    // root splices child
    rules.insert(
        RuleName("root".to_string()),
        YamlRule {
            rule: RuleName("root".to_string()),
            with_params: IndexMap::new(),
            note: None,
            body: vec![make_splice_entry(child_name)],
        },
    );
    let rules_vec: Vec<YamlRule> = rules.into_values().collect();
    YamlConfig {
        version: 0,
        ignore: vec![],
        rules: rules_vec,
        entry: RuleName("root".to_string()),
    }
}
