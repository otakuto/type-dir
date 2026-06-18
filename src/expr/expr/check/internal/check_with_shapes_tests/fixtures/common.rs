use crate::yaml::{
    EntryId, RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern, YamlRule,
    YamlWithShape,
};
use indexmap::IndexMap;

/// Creates a leaf file entry (Exact template).
pub fn file_leaf(template: &str) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact(template.to_string()),
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Creates a dir entry with a regex pattern and a public id.
///
/// The regex is applied to the dir name; the id is assigned to the matched entry.
/// Named captures in the regex become available as capture fields on the binding.
pub fn dir_id_entry(regex: &str, id: &str, inner: Vec<YamlEntry>) -> YamlEntry {
    use crate::yaml::{PatternSpec, RegexPattern};
    YamlEntry {
        id: Some(EntryId(id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPattern(regex.to_string())),
            }),
            body: if inner.is_empty() { None } else { Some(inner) },
            colocated_use_ref: None,
        },
    }
}

/// Creates a rule with a named with param declared as RuleType(rule_name).
///
/// `with_var` is the with variable name; `rule_name` is the rule name (without `rule.` prefix;
/// the prefix is added here to produce the required `rule.<rule_name>` syntax).
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

/// Creates a file entry with a regex pattern and a public id.
///
/// The regex is applied to the file name; the id is assigned to the matched entry.
pub fn file_id_entry(name: &str, id: &str) -> YamlEntry {
    YamlEntry {
        id: Some(EntryId(id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact(name.to_string()),
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Creates a `for x in ${choice.items} { <body> }` for entry.
pub fn for_items_entry(body: Vec<YamlEntry>) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("x".to_string()),
            source: YamlForSource::Expr("${choice.items}".to_string()),
            body,
        },
    }
}

/// Creates an id-bearing alternative entry `- id: <id> / dir: regex: <regex>`.
pub fn dir_alt_entry(id: &str, regex: &str) -> YamlEntry {
    use crate::yaml::{PatternSpec, RegexPattern};
    YamlEntry {
        id: Some(EntryId(id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPattern(regex.to_string())),
            }),
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Creates an id-bearing alternative entry `- id: <id> / file: regex: <regex>`.
pub fn file_alt_entry(id: &str, regex: &str) -> YamlEntry {
    use crate::yaml::{PatternSpec, RegexPattern};
    YamlEntry {
        id: Some(EntryId(id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPattern(regex.to_string())),
            }),
            body: None,
            colocated_use_ref: None,
        },
    }
}

/// Creates an id-bearing `any_of` Sum group: `- any_of: {id: <group_id>, of: <alternatives>}`.
pub fn any_of_sum_entry(group_id: &str, alternatives: Vec<YamlEntry>) -> YamlEntry {
    YamlEntry {
        id: Some(EntryId(group_id.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Choice {
            min: 1,
            max: None,
            body: alternatives,
        },
    }
}

/// Creates a `match: ${<scrutinee>}` entry with arm entries built by the caller.
pub fn match_entry(scrutinee: &str, arms: Vec<YamlEntry>) -> YamlEntry {
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

/// Creates an arm entry `- id: <tag> / rules: <body>` for use inside a `match`.
pub fn arm_entry(tag: &str, body: Vec<YamlEntry>) -> YamlEntry {
    YamlEntry {
        id: Some(EntryId(tag.to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Group {
            body,
            explicit_marker: true,
        },
    }
}

/// Creates a `for <var> in ${<source>} { <body> }` for entry with an arbitrary expression.
pub fn for_expr_entry(iter_var: &str, expr: &str, body: Vec<YamlEntry>) -> YamlEntry {
    YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName(iter_var.to_string()),
            source: YamlForSource::Expr(expr.to_string()),
            body,
        },
    }
}
