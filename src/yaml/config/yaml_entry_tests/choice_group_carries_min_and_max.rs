use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// A `choice:` group yields Choice(min, max, _) with the declared cardinality.
#[test]
fn choice_group_carries_min_and_max() {
    // Arrange
    let yaml = "choice:\nmin: 1\nmax: 2\n::\n  - file: a.rs\n  - file: b.rs\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::Choice { min, max, body, .. } = &entry.kind else {
        panic!("expected Choice but got: {:?}", entry.kind);
    };
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(2));
    assert_eq!(body.len(), 2);
}
