use crate::yaml::config::YamlPattern;

#[test]
fn bare_string_deserializes_as_exact() {
    // Arrange
    let yaml = "foo_bar";

    // Act
    let result: YamlPattern = serde_yaml::from_str(yaml).expect("deserialization failed");

    // Assert
    assert!(matches!(result, YamlPattern::Exact(s) if s == "foo_bar"));
}
