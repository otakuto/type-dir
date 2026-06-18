use crate::yaml::WithShape;

use super::fixtures::*;

/// An empty map `{}` is a valid (field-less) object type.
#[test]
fn empty_map_is_empty_object() {
    // Arrange
    let shape = parse("{}");

    // Act
    let result = shape.to_shape();

    // Assert
    let WithShape::Object(fields) = result.expect("empty map should be an object type") else {
        panic!("expected Object");
    };
    assert!(fields.is_empty(), "empty map should yield no fields");
}
