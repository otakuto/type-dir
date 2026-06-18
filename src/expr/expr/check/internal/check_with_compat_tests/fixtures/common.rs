use crate::yaml::{
    EntryId, PatternSpec, RegexPattern, RuleName, VarName, YamlEntry, YamlEntryKind, YamlPattern,
    YamlRule, YamlWithShape,
};
use indexmap::IndexMap;

/// Creates a file entry with an id using a regex pattern (which has a capture).
pub fn id_file_entry(id: &str, regex: &str) -> YamlEntry {
    let spec = PatternSpec {
        regex: Some(RegexPattern(regex.to_string())),
    };
    YamlEntry {
        id: Some(EntryId(id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Spec(spec),
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Creates a bare rule (splice) entry with `with: var: value`.
pub fn splice_entry(target: &str, with_var: &str, value: &str) -> YamlEntry {
    let mut with_args = IndexMap::new();
    with_args.insert(VarName(with_var.to_string()), value.to_string());
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName(target.to_string()),
            with_args,
            colocated_rules: None,
        },
    }
}

/// Creates a rule with a `RuleType` with declaration using `rule.<rule_name>` syntax.
pub fn rule_with_ruletype_input(
    with_var: &str,
    rule_name: &str,
    entries: Vec<YamlEntry>,
) -> YamlRule {
    let value = serde_yaml::Value::String(format!("rule.{rule_name}"));
    let mut with_params = IndexMap::new();
    with_params.insert(VarName(with_var.to_string()), YamlWithShape(value));
    YamlRule {
        rule: RuleName("".to_string()),
        with_params,
        note: None,
        body: entries,
    }
}

pub fn empty_rule(entries: Vec<YamlEntry>) -> YamlRule {
    YamlRule {
        rule: RuleName("".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: entries,
    }
}
