use super::helpers::{make_dir_rule, make_minimal_yaml, make_splice_entry};
use crate::expr::compile;
use indexmap::IndexMap;

#[test]
fn undefined_rule_reference_returns_err() {
    // Arrange: splice a nonexistent rule within a dir entry's rules
    let inner = make_splice_entry("nonexistent");
    let child = make_dir_rule("child_dir", vec![inner]);
    let yaml = make_minimal_yaml("child", child, IndexMap::new());

    // Act
    let result = compile(yaml);

    // Assert
    assert!(result.is_err());
    let config_errors = result.unwrap_err();
    assert!(!config_errors.0.is_empty());
}
