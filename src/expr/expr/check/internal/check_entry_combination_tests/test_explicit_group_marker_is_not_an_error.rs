use super::super::check_entry_combination;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// An explicit `group:` record-intro (`explicit_marker == true`) with an `id` and a body passes
/// combination validation.
#[test]
fn explicit_group_marker_is_not_an_error() {
    // Arrange: `- group: / id: unit / :: [file foo.rs]` (explicit_marker == true).
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
            explicit_marker: true,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![group]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {:?}", errors);
}
