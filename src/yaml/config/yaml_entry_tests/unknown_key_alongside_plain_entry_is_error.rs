use crate::yaml::config::YamlEntry;

/// An unknown key alongside a plain entry is an error via deny_unknown_fields.
#[test]
fn unknown_key_alongside_plain_entry_is_error() {
    // Arrange: add unknown_key alongside dir
    let yaml = "dir: src\nunknown_key: x\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert
    assert!(
        result.is_err(),
        "unknown key in plain entry did not produce an error: {result:?}"
    );
}
