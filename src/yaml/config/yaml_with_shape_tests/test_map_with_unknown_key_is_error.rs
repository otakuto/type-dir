use crate::yaml::{VarName, WithShape};

use super::fixtures::*;

/// A mapping is an object type; an arbitrary key like `unknown` is a valid field name as long as its
/// value is a valid type. Here the value is the primitive `type.string`.
#[test]
fn object_field_with_valid_type_parses() {
    // Arrange
    let shape = parse("{ unknown: type.string }");

    // Act
    let result = shape.to_shape();

    // Assert
    let WithShape::Object(fields) = result.expect("mapping should be an object type") else {
        panic!("expected Object");
    };
    assert_eq!(
        fields.get(&VarName("unknown".to_string())),
        Some(&WithShape::String)
    );
}
