use crate::yaml::config::YamlWithShape;

fn parse(yaml: &str) -> YamlWithShape {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).expect("yaml parse failed");
    YamlWithShape(value)
}

/// A multi-element array is a parse error.
#[test]
fn test_multi_element_array_is_error() {
    // Arrange
    let shape = parse("[{ a: null }, { b: null }]");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "multiple elements should be an error: {result:?}"
    );
}
