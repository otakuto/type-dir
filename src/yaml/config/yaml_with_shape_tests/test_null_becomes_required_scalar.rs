use super::fixtures::*;

/// `type: null` is now rejected: the ambiguous scalar encoding is removed in favour of an explicit
/// primitive such as `type.string`.
#[test]
fn null_is_error() {
    // Arrange
    let shape = parse("null");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "expected error for `type: null`, got: {result:?}"
    );
    let msg = result.unwrap_err().0;
    assert!(
        msg.contains("no longer supported"),
        "error should explain null is unsupported, got: {msg}"
    );
}
