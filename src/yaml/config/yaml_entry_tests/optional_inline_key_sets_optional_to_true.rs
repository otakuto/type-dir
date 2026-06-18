use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlPattern};

/// An `optional: true` sibling key sets optional=true on the entry.
#[test]
fn optional_inline_key_sets_optional_to_true() {
    // Arrange: place optional: true as a sibling key of dir and splice contents via rules: [use: rule.X]
    let yaml = "dir: tests\noptional: true\n::\n  - use: rule.crate_src_dir\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    assert_eq!(entry.optional, Some(true));
    let YamlEntryKind::Dir { pattern, body, .. } = &entry.kind else {
        panic!("expected Dir but got: {:?}", entry.kind);
    };
    assert!(matches!(pattern, YamlPattern::Exact(s) if s == "tests"));
    let inner = body.as_ref().expect("body is None");
    let YamlEntryKind::Use { rule, .. } = &inner[0].kind else {
        panic!("expected Use but got: {:?}", inner[0].kind);
    };
    assert_eq!(rule.0.as_str(), "crate_src_dir");
}
