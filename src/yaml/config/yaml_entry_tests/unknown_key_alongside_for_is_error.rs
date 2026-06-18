use crate::yaml::config::YamlEntry;

/// An unknown key alongside `for:` is an error via deny_unknown_fields.
#[test]
fn unknown_key_alongside_for_is_error() {
    // Arrange: add an unknown dir alongside for/rules
    let yaml = "for:\n  id: x\n  value: ['a']\n::\n  - file: '${value.x}.rs'\ndir: foo\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert: extra keys should be an error
    assert!(
        result.is_err(),
        "unknown key alongside for did not produce an error: {result:?}"
    );
}
