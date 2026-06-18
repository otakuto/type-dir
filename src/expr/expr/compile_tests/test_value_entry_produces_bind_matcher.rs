use crate::expr::ExprMatcher;
use crate::expr::compile;
use crate::yaml::{RuleName, ValueExpr, VarName, YamlConfig, YamlEntry, YamlEntryKind, YamlRule};
use indexmap::IndexMap;

/// A `value:` entry compiles to an `ExprMatcher::Value` carrying the variable name and value verbatim.
#[test]
fn value_entry_produces_bind_matcher() {
    // Arrange: a root rule whose body is a single scalar value binding.
    let bind_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Value {
            var: VarName("acc".to_string()),
            value: ValueExpr::Scalar("abc".to_string()),
        },
    };
    let root_rule = YamlRule {
        rule: RuleName("root".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![bind_entry],
    };
    let yaml = YamlConfig {
        version: 0,
        ignore: vec![],
        rules: vec![root_rule],
        entry: RuleName("root".to_string()),
    };

    // Act
    let result = compile(yaml);

    // Assert
    assert!(
        result.is_ok(),
        "expected Ok for a rule with a value binding"
    );
    let config = result.unwrap();
    let root = config
        .rules
        .get(&RuleName("root".to_string()))
        .expect("root rule not found");
    assert_eq!(root.rules.len(), 1);
    // The binding is not a capture, so it carries no entry id.
    assert!(root.rules[0].id.is_none());
    let ExprMatcher::Value { var, value } = &root.rules[0].matcher else {
        panic!("expected Value but got a different variant");
    };
    assert_eq!(var.0, "acc");
    assert_eq!(value, &ValueExpr::Scalar("abc".to_string()));
}
