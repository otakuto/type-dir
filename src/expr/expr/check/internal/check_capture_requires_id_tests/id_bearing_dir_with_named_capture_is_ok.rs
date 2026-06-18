use indexmap::IndexMap;

use crate::expr::expr::check::internal::check_capture_requires_id::check_capture_requires_id;
use crate::yaml::{
    EntryId, PatternSpec, RegexPattern, RuleName, YamlEntry, YamlEntryKind, YamlPattern,
};

use super::fixtures::*;

/// An id-bearing dir entry with a named capture is valid (no error).
#[test]
fn id_bearing_dir_with_named_capture_is_ok() {
    // Arrange
    let entry = YamlEntry {
        id: Some(EntryId("mydir".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPattern(r"^(?<stem>[a-z]+)$".to_string())),
            }),
            body: None,
            colocated_use_ref: None,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("test_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_capture_requires_id(&rules);

    // Assert
    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}
