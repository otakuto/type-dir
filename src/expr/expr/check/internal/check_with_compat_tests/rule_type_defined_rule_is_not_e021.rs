use crate::error::SemanticError;
use crate::expr::expr::check::internal::check_with_compat::check_with_compat;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::{RuleName, VarName, YamlRule, YamlWithShape};

use super::fixtures::empty_rule;
use indexmap::IndexMap;

/// When a with param is declared as `RuleType(R)` (via `rule.feature_dir`) and R is defined, no E021.
#[test]
fn rule_type_defined_rule_is_not_e021() {
    // Arrange: rule declares with x as type "rule.feature_dir" which is defined
    let value: serde_yaml::Value =
        serde_yaml::from_str("rule.feature_dir").expect("yaml parse failed");
    let mut with_params = IndexMap::new();
    with_params.insert(VarName("x".to_string()), YamlWithShape(value));
    let consumer_rule = YamlRule {
        rule: RuleName("consumer".to_string()),
        with_params,
        note: None,
        body: vec![],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("consumer".to_string()), consumer_rule);
    rules.insert(RuleName("feature_dir".to_string()), empty_rule(vec![]));

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_compat(&rules, &id_shapes);

    // Assert: no errors
    let e021_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, SemanticError::UndefinedShapeRule { .. }))
        .collect();
    assert!(e021_errors.is_empty(), "unexpected E021: {errors:?}");
}
