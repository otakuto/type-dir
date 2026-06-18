use super::helpers::{make_dir_rule, make_minimal_yaml};
use crate::expr::compile;
use indexmap::IndexMap;

#[test]
fn valid_yaml_config_builds_config_expr() {
    // Arrange: root splices child, and child describes the src dir
    let child = make_dir_rule("src", vec![]);
    let yaml = make_minimal_yaml("child", child, IndexMap::new());

    // Act
    let result = compile(yaml);

    // Assert
    let config = result.expect("Err returned for valid config");
    assert_eq!(config.entry.0, "root");
    assert_eq!(config.rules.len(), 2); // root + child
}
