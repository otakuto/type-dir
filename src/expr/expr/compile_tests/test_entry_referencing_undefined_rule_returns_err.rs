use crate::expr::compile;
use crate::yaml::{RuleName, YamlConfig};

#[test]
fn entry_referencing_undefined_rule_returns_err() {
    // Arrange: entry references a rule name that is not defined (scalar)
    let yaml = YamlConfig {
        version: 0,
        ignore: vec![],
        rules: vec![],
        entry: RuleName("nonexistent_root".to_string()),
    };

    // Act
    let result = compile(yaml);

    // Assert
    assert!(result.is_err());
    let config_errors = result.unwrap_err();
    assert!(!config_errors.0.is_empty());
}
