use super::fixtures::*;

/// A bare rule name (no `rule.` prefix) is now rejected as a parse error.
#[test]
fn bare_rule_name_is_error() {
    // Arrange
    let shape = parse("feature_dir");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "expected error for bare rule name, got: {result:?}"
    );
}
