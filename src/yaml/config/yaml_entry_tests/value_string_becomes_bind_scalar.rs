use crate::yaml::ValueExpr;
use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// `- id: acc / value: 'abc'` parses as a `Value` with a scalar value and no entry id.
#[test]
fn value_string_becomes_bind_scalar() {
    // Arrange
    let yaml = "id: acc\nvalue: 'abc'\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    // The binding name lives on the kind, not on `entry.id` (it is not a capture).
    assert!(entry.id.is_none());
    let YamlEntryKind::Value { var, value } = &entry.kind else {
        panic!("expected Value but got: {:?}", entry.kind);
    };
    assert_eq!(var.0, "acc");
    assert_eq!(value, &ValueExpr::Scalar("abc".to_string()));
}
