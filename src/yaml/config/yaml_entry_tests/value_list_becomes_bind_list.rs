use crate::yaml::ValueExpr;
use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// `- id: names / value: ['a', 'b', 'c']` parses as a `Value` with a list value.
#[test]
fn value_list_becomes_bind_list() {
    // Arrange
    let yaml = "id: names\nvalue: ['a', 'b', 'c']\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::Value { var, value } = &entry.kind else {
        panic!("expected Bind but got: {:?}", entry.kind);
    };
    assert_eq!(var.0, "names");
    assert_eq!(
        value,
        &ValueExpr::List(vec!["a".to_string(), "b".to_string(), "c".to_string(),])
    );
}
