use crate::yaml::config::YamlEntry;

/// Placing `optional:` alongside a group (one_of) is an error.
#[test]
fn optional_alongside_one_of_is_error() {
    // Arrange: coexistence of optional: true with one_of group is prohibited (only diag is allowed)
    let yaml = "one_of:\n::\n  - file: a.rs\n  - file: b.rs\noptional: true\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert: coexistence with group should be an error
    assert!(
        result.is_err(),
        "optional alongside one_of did not produce an error: {result:?}"
    );
}
