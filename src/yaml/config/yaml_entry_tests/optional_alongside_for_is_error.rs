use crate::yaml::config::YamlEntry;

/// Placing `optional:` alongside a `for:` entry is an error.
#[test]
fn optional_alongside_for_is_error() {
    // Arrange: coexistence of optional with for/rules is prohibited (ForRepr has deny_unknown_fields)
    let yaml = "for:\n  id: x\n  value: ['a']\n::\n  - file: '${value.x}.rs'\noptional: true\n";

    // Act
    let result: Result<YamlEntry, _> = serde_yaml::from_str(yaml);

    // Assert
    assert!(
        result.is_err(),
        "optional alongside for did not produce an error: {result:?}"
    );
}
