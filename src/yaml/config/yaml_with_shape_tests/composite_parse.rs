use crate::yaml::{VarName, WithShape};

use super::fixtures::*;

/// `[type.string]` parses to an array of strings.
#[test]
fn array_of_string_parses() {
    // Arrange / Act
    let result = parse("[type.string]").to_shape().unwrap();

    // Assert
    assert_eq!(result, WithShape::Array(Box::new(WithShape::String)));
}

/// `[[type.number]]` parses to a nested (2-D) array of numbers.
#[test]
fn nested_array_parses() {
    // Arrange / Act
    let result = parse("[[type.number]]").to_shape().unwrap();

    // Assert
    assert_eq!(
        result,
        WithShape::Array(Box::new(WithShape::Array(Box::new(WithShape::Number))))
    );
}

/// `{a: type.string, b: [[type.number]]}` parses to an object with the two declared fields, in order.
#[test]
fn object_with_fields_parses() {
    // Arrange / Act
    let result = parse("{a: type.string, b: [[type.number]]}")
        .to_shape()
        .unwrap();

    // Assert
    let WithShape::Object(fields) = result else {
        panic!("expected Object, got {result:?}");
    };
    let keys: Vec<&String> = fields.keys().map(|v| &v.0).collect();
    assert_eq!(keys, vec!["a", "b"], "field order is preserved");
    assert_eq!(
        fields.get(&VarName("a".to_string())),
        Some(&WithShape::String)
    );
    assert_eq!(
        fields.get(&VarName("b".to_string())),
        Some(&WithShape::Array(Box::new(WithShape::Array(Box::new(
            WithShape::Number
        )))))
    );
}
