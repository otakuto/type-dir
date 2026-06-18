use crate::yaml::{RuleName, WithShape};

use super::fixtures::*;

/// A `rule.<name>` string (with prefix) is parsed as a RuleType reference with an empty path.
#[test]
fn test_string_becomes_rule_type() {
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
