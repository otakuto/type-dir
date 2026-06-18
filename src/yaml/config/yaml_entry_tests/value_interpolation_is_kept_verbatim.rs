use crate::yaml::ValueExpr;
use crate::yaml::config::{YamlEntry, YamlEntryKind};

/// `value: '${with.prefix}-${dir.name}'` keeps the template verbatim (interpolation happens at
/// enforcement time, not at parse time).
#[test]
fn value_interpolation_is_kept_verbatim() {
    // Arrange
    let yaml = "id: acc2\nvalue: '${with.prefix}-${dir.name}'\n";

    // Act
    let entry: YamlEntry = serde_yaml::from_str(yaml).unwrap();

    // Assert
    let YamlEntryKind::Value { var, value } = &entry.kind else {
        panic!("expected Value but got: {:?}", entry.kind);
    };
    assert_eq!(var.0, "acc2");
    assert_eq!(
        value,
        &ValueExpr::Scalar("${with.prefix}-${dir.name}".to_string())
    );
}
