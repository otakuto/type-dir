use super::fixtures::*;

/// An object type whose field value is `null` is rejected: each field must declare an explicit type
/// (`null` is no longer a valid type anywhere).
#[test]
fn object_field_with_null_type_is_error() {
    // Arrange
    let shape = parse("{ default: null }");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "expected error: a null field type is unsupported"
    );
}
