use crate::yaml::config::YamlEntry;

/// `value:` coexisting with `dir:` is a parse error: `ValueRepr` uses `deny_unknown_fields`, so any
/// key other than `id`/`value` is rejected, keeping `value:` exclusive with all other entry kinds.
#[test]
fn value_alongside_dir_is_error() {
    // Arrange
    let yaml = "id: acc\nvalue: 'abc'\ndir: feature\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert
    assert!(result.is_err());
}
