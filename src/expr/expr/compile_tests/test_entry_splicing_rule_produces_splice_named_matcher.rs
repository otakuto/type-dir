use super::helpers::{make_dir_rule, make_splice_entry};
use crate::expr::ExprMatcher;
use crate::expr::compile;
use crate::yaml::{RuleName, YamlConfig, YamlRule};
use indexmap::IndexMap;

#[test]
fn entry_splicing_rule_produces_splice_named_matcher() {
    // Arrange: root splices child
    let child_name = RuleName("child".to_string());
    let child_rule = {
        let mut r = make_dir_rule("child_dir", vec![]);
        r.rule = child_name.clone();
        r
    };
    let root_rule = YamlRule {
        rule: RuleName("root".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![make_splice_entry("child")],
    };
    let yaml = YamlConfig {
        version: 0,
        ignore: vec![],
        rules: vec![child_rule, root_rule],
        entry: RuleName("root".to_string()),
    };

    // Act
    let result = compile(yaml);

    // Assert
    assert!(result.is_ok(), "expected Ok for root rule that splices");
    let config = result.unwrap();
    assert_eq!(config.entry.0, "root");
    let root_rule = config
        .rules
        .get(&RuleName("root".to_string()))
        .expect("root rule not found");
    assert_eq!(root_rule.rules.len(), 1);
    let ExprMatcher::Use { rule, .. } = &root_rule.rules[0].matcher else {
        panic!("expected Use but got a different variant");
    };
    assert_eq!(rule, &child_name);
}
