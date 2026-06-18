use super::fixtures::*;

/// A one-element array `[T]` is a valid array type, but here the element `{ stem: null }` has a null
/// field type, which is rejected (the element type must itself be valid).
#[test]
fn array_with_invalid_element_is_error() {
    // Arrange
    let shape = parse("[{ stem: null }]");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "expected error: the array element type is invalid"
    );
}
