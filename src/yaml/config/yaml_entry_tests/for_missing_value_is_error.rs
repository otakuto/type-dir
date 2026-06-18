use crate::yaml::config::YamlEntry;

/// Missing `value:` inside the `for: {id, value}` map produces a clear error.
#[test]
fn for_missing_value_is_error() {
    // Arrange: for entry whose `for:` map omits the required `value` key
    let yaml = "for:\n  id: x\n::\n  - file: '${value.x}.rs'\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert: missing required field should be an error
    assert!(
        result.is_err(),
        "missing value did not produce an error: {result:?}"
    );
}
