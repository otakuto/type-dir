use crate::yaml::{
    RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern, YamlRule,
};
use indexmap::IndexMap;

pub fn splice_entry(rule: &str) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName(rule.to_string()),
            with_args: IndexMap::new(),
            colocated_rules: None,
        },
    }
}

pub fn dir_entry_with(name: &str, inner: Vec<YamlEntry>) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact(name.to_string()),
            body: if inner.is_empty() { None } else { Some(inner) },
            colocated_use_ref: None,
        },
    }
}

pub fn for_entry_with(var: &str, source: &str, inner: Vec<YamlEntry>) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName(var.to_string()),
            source: YamlForSource::Expr(source.to_string()),
            body: inner,
        },
    }
}

pub fn rule(entries: Vec<YamlEntry>) -> YamlRule {
    YamlRule {
        rule: RuleName("".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: entries,
    }
}
