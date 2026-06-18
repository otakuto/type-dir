use crate::error::SemanticError;
use crate::expr::compile;
use crate::yaml::{RuleName, YamlConfig, YamlRule};
use indexmap::IndexMap;

#[test]
fn duplicate_rule_name_returns_err() {
    // Arrange: two rule definitions in the top-level `rules:` list share the name `dup`.
    let dup = || YamlRule {
        rule: RuleName("dup".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![],
    };
    let yaml = YamlConfig {
        version: 0,
        ignore: vec![],
        rules: vec![dup(), dup()],
        entry: RuleName("dup".to_string()),
    };

    // Act
    let result = compile(yaml);

    // Assert
    let config_errors = result.unwrap_err();
    assert!(
        config_errors.0.iter().any(|e| matches!(
            e,
            SemanticError::DuplicateRule { rule } if rule == "dup"
        )),
        "expected DuplicateRule for `dup`, got: {:?}",
        config_errors.0
    );
}
