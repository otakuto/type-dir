use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// An explicit `group:` marker yields `YamlEntryKind::Group` with `explicit_marker == true`,
/// carrying its `::` body and the sibling `id:`.
#[test]
fn group_marker_sets_group_kind() {
    // Arrange: a value-less `group:` marker with a sibling `id:` and a `::` body.
    let yaml = "group:\nid: aaa\n::\n  - file: foo.rs\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert!(matches!(entry.id.as_ref(), Some(id) if id.0 == "aaa"));
    let YamlEntryKind::Group {
        body,
        explicit_marker,
    } = &entry.kind
    else {
        panic!("expected Group but got: {:?}", entry.kind);
    };
    assert!(
        explicit_marker,
        "explicit `group:` marker must set explicit_marker"
    );
    assert_eq!(body.len(), 1);
}

/// An entry with `::` but no dir/file/use and no `group:` marker (the legacy implicit form) still
/// parses, but is produced with `explicit_marker == false` so the semantic check can reject it.
#[test]
fn implicit_group_parses_without_marker() {
    // Arrange: id + `::` only (no dir/file/use, no group: marker).
    let yaml = "id: aaa\n::\n  - file: foo.rs\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::Group {
        explicit_marker, ..
    } = &entry.kind
    else {
        panic!("expected Group but got: {:?}", entry.kind);
    };
    assert!(
        !explicit_marker,
        "the implicit form must set explicit_marker == false"
    );
}
