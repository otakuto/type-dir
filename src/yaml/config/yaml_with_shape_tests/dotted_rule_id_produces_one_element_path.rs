use crate::yaml::{EntryId, RuleName, WithShape};

use super::fixtures::*;

/// A dotted string `rule.feature_dir.dir.feature_name` produces `path: [feature_name]`.
/// The `<kind>.<name>` pair is required; only the name is kept in the path.
#[test]
fn dotted_rule_id_produces_one_element_path() {
    // Arrange
    let shape = parse("rule.feature_dir.dir.feature_name");

    // Act
    let result = shape.to_shape();

    // Assert
    assert_eq!(
        result.unwrap(),
        WithShape::RuleType {
            rule: RuleName("feature_dir".to_string()),
            path: vec![EntryId("feature_name".to_string())],
        }
    );
}
