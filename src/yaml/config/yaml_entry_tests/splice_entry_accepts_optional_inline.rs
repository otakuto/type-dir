use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// A `use: rule.<name>` splice entry can also have `optional: true` as a sibling key.
#[test]
fn splice_entry_accepts_optional_inline() {
    // Arrange
    let yaml = "use: rule.integration_tests_dir\noptional: true\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.optional, Some(true));
    let YamlEntryKind::Use { rule, .. } = &entry.kind else {
        panic!("expected Use but got: {:?}", entry.kind);
    };
    assert_eq!(rule.0.as_str(), "integration_tests_dir");
}
