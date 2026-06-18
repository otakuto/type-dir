use super::helpers::{make_dir_rule, make_minimal_yaml};
use crate::expr::compile;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};
use indexmap::IndexMap;

#[test]
fn dir_entry_with_colocated_rule_returns_err() {
    // Arrange: place rule alongside a dir entry (should write rules: [use: rule.X] instead)
    let mut extra_rules = IndexMap::new();
    extra_rules.insert(
        RuleName("some_rule".to_string()),
        YamlRule {
            rule: RuleName("some_rule".to_string()),
            with_params: IndexMap::new(),
            note: None,
            body: vec![],
        },
    );
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("foo".to_string()),
            body: None,
            colocated_use_ref: Some(RuleName("some_rule".to_string())),
        },
    };
    let child = make_dir_rule("child_dir", vec![entry]);
    let yaml = make_minimal_yaml("child", child, extra_rules);

    // Act
    let result = compile(yaml);

    // Assert
    assert!(result.is_err());
    let config_errors = result.unwrap_err();
    assert!(!config_errors.0.is_empty());
}
