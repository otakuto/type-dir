use super::fixtures::*;

/// `{ default: 'src' }` is an object type whose field value `'src'` is not a valid type string, so
/// it is rejected. (The former `{ default: ... }` scalar-default form no longer has special meaning;
/// a mapping is an object type and each value must itself be a type.)
#[test]
fn object_field_with_invalid_type_is_error() {
    // Arrange
    let shape = parse("{ default: 'src' }");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "expected error: `src` is not a valid field type"
    );
}
