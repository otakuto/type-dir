use crate::yaml::config::YamlEntry;

/// Map form `count: {min, max}` is an error (removed).
#[test]
fn count_map_form_is_error() {
    // Arrange: the old map form for count is not accepted
    let yaml = "file: a.rs\ncount:\n  min: 1\n  max: 3\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert: map form is a type error (count accepts usize scalar only)
    assert!(
        result.is_err(),
        "count map form did not produce an error: {result:?}"
    );
}
