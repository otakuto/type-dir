use crate::yaml::{
    RuleName, ValueExpr, VarName, YamlEntry, YamlEntryKind, YamlRule, YamlWithShape,
};
use indexmap::IndexMap;

pub fn dir_entry(pattern: crate::yaml::YamlPattern) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern,
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Builds a `value:` binding entry (`- id: <var> / value: ...`).
pub fn bind_entry(var: &str, value: ValueExpr) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Value {
            var: VarName(var.to_string()),
            value,
        },
    }
}

pub fn rule_with(with_vars: &[&str], entries: Vec<YamlEntry>) -> YamlRule {
    let mut with_params = IndexMap::new();
    for key in with_vars {
        // null = required scalar equivalent (YamlWithShape wraps serde_yaml::Value::Null)
        with_params.insert(
            VarName(key.to_string()),
            YamlWithShape(serde_yaml::Value::Null),
        );
    }
    YamlRule {
        rule: RuleName("".to_string()),
        with_params,
        note: None,
        body: entries,
    }
}
