use crate::yaml::config::YamlEntry;

/// `min: n` and `max: m` can be specified independently.
#[test]
fn min_max_individual_fields_are_preserved() {
    // Arrange
    let yaml = "file: a.rs\nmin: 2\nmax: 5\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.min, Some(2));
    assert_eq!(entry.max, Some(5));
}
