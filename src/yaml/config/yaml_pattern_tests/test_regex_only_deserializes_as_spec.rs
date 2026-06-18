use crate::yaml::config::YamlPattern;

#[test]
fn regex_only_deserializes_as_spec() {
    // Arrange
    let yaml = "regex: '^x$'\n";

    // Act
    let result: YamlPattern = serde_yaml::from_str(yaml).expect("deserialization failed");

    // Assert
    let YamlPattern::Spec(spec) = result else {
        panic!("expected Spec but Exact was returned");
    };
    assert!(spec.regex.is_some());
}
