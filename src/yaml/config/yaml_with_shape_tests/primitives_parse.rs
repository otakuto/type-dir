use crate::yaml::WithShape;

use super::fixtures::*;

/// `type.string` / `type.number` / `type.bool` parse to the corresponding primitive shapes.
#[test]
fn primitives_parse() {
    // Arrange / Act / Assert
    assert_eq!(parse("type.string").to_shape().unwrap(), WithShape::String);
    assert_eq!(parse("type.number").to_shape().unwrap(), WithShape::Number);
    assert_eq!(parse("type.bool").to_shape().unwrap(), WithShape::Bool);
}

/// An unknown `type.<x>` primitive is rejected.
#[test]
fn unknown_primitive_is_error() {
    // Arrange
    let shape = parse("type.float");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(result.is_err(), "expected error for unknown primitive");
}
