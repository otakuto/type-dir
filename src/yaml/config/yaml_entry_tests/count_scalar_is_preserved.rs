use crate::yaml::config::YamlEntry;

/// `count: n` (scalar) is retained as count=Some(n).
#[test]
fn count_scalar_is_preserved() {
    // Arrange
    let yaml = "file: a.rs\ncount: 3\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.count, Some(3));
}
