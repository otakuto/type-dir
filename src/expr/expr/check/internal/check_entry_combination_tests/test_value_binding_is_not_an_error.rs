use super::super::check_entry_combination;
use crate::yaml::{RuleName, ValueExpr, VarName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// A standalone `value:` binding entry passes combination validation (exclusivity with other entry
/// keys is enforced at parse time via `deny_unknown_fields`).
#[test]
fn value_binding_is_not_an_error() {
    // Arrange: a single value binding entry.
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Value {
            var: VarName("acc".to_string()),
            value: ValueExpr::Scalar("abc".to_string()),
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {:?}", errors);
}
