use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlPattern};

/// A `any_of:` group yields Choice(min=1, max=None, _).
#[test]
fn any_of_group_sets_group_kind_any_of() {
    // Arrange
    let yaml = "any_of:\n::\n  - file: a.rs\n  - file: b.rs\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(entry.optional.is_none());
    let YamlEntryKind::Choice { min, max, body, .. } = &entry.kind else {
        panic!("expected Choice but got: {:?}", entry.kind);
    };
    assert_eq!(*min, 1);
    assert_eq!(*max, None);
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
    assert!(matches!(p0, YamlPattern::Exact(s) if s == "a.rs"));
    assert!(matches!(p1, YamlPattern::Exact(s) if s == "b.rs"));
}
