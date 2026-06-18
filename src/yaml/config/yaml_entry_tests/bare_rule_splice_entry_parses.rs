use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// A `use: rule.<name>` splice entry parses with optional=None.
#[test]
fn bare_rule_splice_entry_parses() {
    // Arrange
    let yaml = "use: rule.crate_src_dir\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(entry.optional.is_none());
    let YamlEntryKind::Use { rule, .. } = &entry.kind else {
        panic!("expected Use but got: {:?}", entry.kind);
    };
    assert_eq!(rule.0.as_str(), "crate_src_dir");
}
