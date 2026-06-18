use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// Omitting min/max in `choice:` defaults to min=0, max=None (∞).
#[test]
fn choice_omitted_min_max_defaults_to_zero_and_none() {
    // Arrange: specify only of
    let yaml = "choice:\n::\n  - file: a.rs\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::Choice { min, max, .. } = &entry.kind else {
        panic!("expected Choice but got: {:?}", entry.kind);
    };
    assert_eq!(*min, 0);
    assert_eq!(*max, None);
}
