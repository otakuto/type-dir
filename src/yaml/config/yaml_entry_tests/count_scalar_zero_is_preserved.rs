use crate::yaml::config::YamlEntry;

/// `count: 0` is retained as scalar 0.
#[test]
fn count_scalar_zero_is_preserved() {
    // Arrange
    let yaml = "file: a.rs\ncount: 0\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.count, Some(0));
}
