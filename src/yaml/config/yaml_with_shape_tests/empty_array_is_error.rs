use super::fixtures::*;

/// An empty array `[]` is now rejected (arrays are no longer supported as input shapes).
#[test]
fn empty_array_is_error() {
    // Arrange
    let shape = parse("[]");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(result.is_err(), "expected error for array input shape");
}
