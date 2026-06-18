use crate::yaml::config::YamlEntry;

/// `optional: false` results in Some(false).
#[test]
fn optional_inline_false_sets_optional_to_false() {
    // Arrange
    let yaml = "file: README.md\noptional: false\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.optional, Some(false));
}
