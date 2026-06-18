use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlPattern};

/// An `optional: true` sibling key sets optional=Some(true).
#[test]
fn optional_inline_true_sets_optional_to_true() {
    // Arrange: place optional: true as a sibling key of file
    let yaml = "file: README.md\noptional: true\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.optional, Some(true));
    let YamlEntryKind::File { pattern, .. } = &entry.kind else {
        panic!("expected File but got: {:?}", entry.kind);
    };
    assert!(matches!(pattern, YamlPattern::Exact(s) if s == "README.md"));
}
