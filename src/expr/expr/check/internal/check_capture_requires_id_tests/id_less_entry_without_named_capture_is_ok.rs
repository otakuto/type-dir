use indexmap::IndexMap;

use crate::expr::expr::check::internal::check_capture_requires_id::check_capture_requires_id;
use crate::yaml::{PatternSpec, RegexPattern, RuleName, YamlEntry, YamlEntryKind, YamlPattern};

use super::fixtures::*;

/// An id-less entry whose regex has no named captures is valid (no error).
#[test]
fn id_less_entry_without_named_capture_is_ok() {
    // Arrange: regex with no named groups (only unnamed groups)
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPattern(r"^[a-z]+\.rs$".to_string())),
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
