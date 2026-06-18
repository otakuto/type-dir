use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// An implicit record-intro group (`::` body with an `id` but no dir/file/use and no `group:`
/// marker, i.e. `explicit_marker == false`) is rejected with `ImplicitGroup` — the author must
/// declare it with the explicit `group:` keyword.
#[test]
fn implicit_group_without_marker_is_error() {
    // Arrange: `- id: unit / :: [file foo.rs]` with explicit_marker == false (the legacy form).
    let child = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("foo.rs".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let group = YamlEntry {
        id: Some(EntryId("unit".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Group {
            body: vec![child],
            explicit_marker: false,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![group]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    assert!(
        matches!(&errors[0], SemanticError::ImplicitGroup { .. }),
        "unexpected: {:?}",
        errors[0]
    );
}
