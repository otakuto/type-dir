use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlPattern};

/// A `one_of:` group yields Choice(min=1, max=Some(1), _).
#[test]
fn one_of_group_sets_group_kind_one_of() {
    // Arrange
    let yaml = "one_of:\n::\n  - file: config.toml\n  - file: config.yaml\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(entry.optional.is_none());
    let YamlEntryKind::Choice { min, max, body, .. } = &entry.kind else {
        panic!("expected Choice but got: {:?}", entry.kind);
    };
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(1));
    assert_eq!(body.len(), 2);
    let (YamlEntryKind::Dir { pattern: p0, .. } | YamlEntryKind::File { pattern: p0, .. }) =
        &body[0].kind
    else {
        panic!("expected Dir/File alt[0]");
    };
    let (YamlEntryKind::Dir { pattern: p1, .. } | YamlEntryKind::File { pattern: p1, .. }) =
        &body[1].kind
    else {
        panic!("expected Dir/File alt[1]");
    };
    assert!(matches!(p0, YamlPattern::Exact(s) if s == "config.toml"));
    assert!(matches!(p1, YamlPattern::Exact(s) if s == "config.yaml"));
}
