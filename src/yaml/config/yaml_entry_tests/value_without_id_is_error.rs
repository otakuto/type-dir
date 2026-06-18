use crate::yaml::config::YamlEntry;

/// A `value:` entry without `id:` is a parse error (`id` is the bound variable name and is required).
#[test]
fn value_without_id_is_error() {
    // Arrange
    let yaml = "value: 'abc'\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert
    assert!(result.is_err());
}
