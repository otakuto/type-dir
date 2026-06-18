use crate::yaml::{RuleName, WithShape};

use super::fixtures::*;

/// A `rule.<name>` string (with prefix, no dot after name) produces `RuleType { rule, path: [] }`.
#[test]
fn rule_prefixed_bare_name_produces_empty_path() {
    // Arrange
    let shape = parse("rule.feature_dir");

    // Act
    let result = shape.to_shape();

    // Assert
    assert_eq!(
        result.unwrap(),
        WithShape::RuleType {
            rule: RuleName("feature_dir".to_string()),
            path: vec![],
        }
    );
}
