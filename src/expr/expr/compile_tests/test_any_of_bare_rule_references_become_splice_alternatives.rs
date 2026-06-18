use super::helpers::{make_dir_rule, make_minimal_yaml, make_splice_entry};
use crate::expr::compile;
use crate::expr::{ExprMatcher, ExprSubtree};
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};
use indexmap::IndexMap;

#[test]
fn any_of_bare_rule_references_become_splice_alternatives() {
    // Arrange: validate parent dir contents using any_of with two bare rule references.
    let mut extra_rules = IndexMap::new();
    extra_rules.insert(
        RuleName("resource_dir".to_string()),
        YamlRule {
            rule: RuleName("resource_dir".to_string()),
            with_params: IndexMap::new(),
            note: None,
            body: vec![],
        },
    );
    extra_rules.insert(
        RuleName("resource_group_dir".to_string()),
        YamlRule {
            rule: RuleName("resource_group_dir".to_string()),
            with_params: IndexMap::new(),
            note: None,
            body: vec![],
        },
    );
    // each alternative of any_of is a bare rule reference (no dir/file)
    let alt1 = make_splice_entry("resource_dir");
    let alt2 = make_splice_entry("resource_group_dir");
    // entry that has an any_of group inline as the contents of dir: envs
    let group_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Choice {
            min: 1,
            max: None,
            body: vec![alt1, alt2],
        },
    };
    let envs_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("envs".to_string()),
            body: Some(vec![group_entry]),
            colocated_use_ref: None,
        },
    };
    let child = make_dir_rule("child_dir", vec![envs_entry]);
    let yaml = make_minimal_yaml("child", child, extra_rules);

    // Act
    let result = compile(yaml);

    // Assert
    let config = result.expect("Err returned for any_of bare rule references");
    let child_rule = config
        .rules
        .get(&RuleName("child".to_string()))
        .expect("child rule not found");
    // child_rule.rules[0] is the child_dir entry (Dir + Inline)
    let (ExprMatcher::Dir { subtree, .. } | ExprMatcher::File { subtree, .. }) =
        &child_rule.rules[0].matcher
    else {
        panic!("expected Dir/File for child_dir entry");
    };
    let ExprSubtree::Inline(outer_inline) = subtree else {
        panic!("expected Inline for child_dir subtree");
    };
    // outer_inline[0] is the envs entry (Dir + Inline)
    let (ExprMatcher::Dir { subtree, .. } | ExprMatcher::File { subtree, .. }) =
        &outer_inline[0].matcher
    else {
        panic!("expected Dir/File for envs entry");
    };
    let ExprSubtree::Inline(inline) = subtree else {
        panic!("expected Inline for envs subtree");
    };
    // inline[0] is the any_of Group entry
    let ExprMatcher::Choice { body, .. } = &inline[0].matcher else {
        panic!("expected Group");
    };
    assert_eq!(body.len(), 2);
    // each alternative should be a Splice
    let ExprMatcher::Use { rule: r1, .. } = &body[0].matcher else {
        panic!("expected Splice for alternative[0]");
    };
    let ExprMatcher::Use { rule: r2, .. } = &body[1].matcher else {
        panic!("expected Splice for alternative[1]");
    };
    assert_eq!(r1.0, "resource_dir");
    assert_eq!(r2.0, "resource_group_dir");
}
