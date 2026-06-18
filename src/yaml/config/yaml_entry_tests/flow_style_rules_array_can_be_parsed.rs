use crate::yaml::config::{YamlEntry, YamlEntryKind, YamlPattern};

/// Flow-style `rules: [{ use: rule.X }]` (explicit flow mapping) can be parsed.
#[test]
fn flow_style_rules_array_can_be_parsed() {
    // Arrange: write dir entry contents in flow form rules: [{ use: rule.X }]
    let yaml = "dir: foo\n:: [{ use: rule.crate_src_dir }]\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::Dir { pattern, body, .. } = &entry.kind else {
        panic!("expected Dir but got: {:?}", entry.kind);
    };
    assert!(matches!(pattern, YamlPattern::Exact(s) if s == "foo"));
    let inner = body.as_ref().expect("body is None");
    assert_eq!(inner.len(), 1);
    let YamlEntryKind::Use { rule, .. } = &inner[0].kind else {
        panic!("expected Use but got: {:?}", inner[0].kind);
    };
    assert_eq!(rule.0.as_str(), "crate_src_dir");
}
