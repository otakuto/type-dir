use crate::error::SemanticError;
use crate::expr::expr::check::internal::check_with_compat::check_with_compat;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::{RuleName, VarName, YamlRule, YamlWithShape};
use indexmap::IndexMap;

/// When a with param is declared as `RuleType(R)` (via `rule.undefined_dir`) and R is not defined,
/// E021 is emitted.
#[test]
fn rule_type_undefined_rule_is_e021() {
    // Arrange: rule declares with x as type "rule.undefined_dir" which does not exist
    let value: serde_yaml::Value =
        serde_yaml::from_str("rule.undefined_dir").expect("yaml parse failed");
    let mut with_params = IndexMap::new();
    with_params.insert(VarName("x".to_string()), YamlWithShape(value));
    let rule = YamlRule {
        rule: RuleName("consumer".to_string()),
        with_params,
        note: None,
        body: vec![],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("consumer".to_string()), rule);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_compat(&rules, &id_shapes);

    // Assert: one UndefinedShapeRule (E021)
    assert_eq!(errors.len(), 1, "expected 1 E021: {errors:?}");
    match &errors[0] {
        SemanticError::UndefinedShapeRule {
            rule,
            with,
            ref_rule,
        } => {
            assert_eq!(rule.as_str(), "consumer");
            assert_eq!(with.as_str(), "x");
            assert_eq!(ref_rule.as_str(), "undefined_dir");
        }
        _ => panic!("expected UndefinedShapeRule: {:?}", errors[0]),
    }
}
