use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlForSource};

/// The `for: {id, value}` map plus `rules:` parses as a for entry (happy path).
#[test]
fn for_three_field_set_becomes_for_entry() {
    // Arrange
    let yaml = "for:\n  id: x\n  value: ['a', 'b']\n::\n  - file: '${value.x}.rs'\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::For { var, source, body } = &entry.kind else {
        panic!("expected For but got: {:?}", entry.kind);
    };
    assert_eq!(var.0, "x");
    assert!(
        matches!(source, YamlForSource::Literal(v) if v == &["a".to_string(), "b".to_string()])
    );
    assert_eq!(body.len(), 1);
}
