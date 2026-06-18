use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlPattern};

/// A plain entry without a wrapper has optional=None.
#[test]
fn plain_entry_has_optional_none() {
    // Arrange
    let yaml = "dir: src\n::\n  - use: rule.crate_src_dir\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(entry.optional.is_none());
    let (YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. }) = &entry.kind
    else {
        panic!("expected Dir/File but got: {:?}", entry.kind);
    };
    assert!(matches!(pattern, YamlPattern::Exact(s) if s == "src"));
}
