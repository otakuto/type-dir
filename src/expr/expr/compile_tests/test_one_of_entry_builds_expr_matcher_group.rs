use super::helpers::{make_dir_rule, make_minimal_yaml};
use crate::expr::compile;
use crate::expr::{ExprMatcher, ExprSubtree};
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

#[test]
fn one_of_entry_builds_expr_matcher_group() {
    // Arrange
    let alt1 = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("config.toml".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let alt2 = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("config.yaml".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let group_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt1, alt2],
        },
    };
    let child = make_dir_rule("child_dir", vec![group_entry]);
    let yaml = make_minimal_yaml("child", child, IndexMap::new());

    // Act
    let result = compile(yaml);

    // Assert
    let config = result.expect("Err returned for one_of entry");
    assert_eq!(config.entry.0, "root");
    let child_rule = config
        .rules
        .get(&RuleName("child".to_string()))
        .expect("child rule not found");
    // child body is a single dir entry (Dir + Inline)
    let (ExprMatcher::Dir { subtree, .. } | ExprMatcher::File { subtree, .. }) =
        &child_rule.rules[0].matcher
    else {
        panic!("expected Dir/File for child_dir entry");
    };
    let ExprSubtree::Inline(inline) = subtree else {
        panic!("expected Inline for child_dir subtree");
    };
    let ExprMatcher::Choice { min, max, body, .. } = &inline[0].matcher else {
        panic!(
            "expected ExprMatcher::Choice but got a different variant: {:?}",
            inline[0].matcher
        );
    };
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(1));
    assert_eq!(body.len(), 2);
}
