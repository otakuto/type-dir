use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::expr::expr::check::internal::check_capture_requires_id::check_capture_requires_id;
use crate::yaml::{PatternSpec, RegexPattern, RuleName, YamlEntry, YamlEntryKind, YamlPattern};

use super::fixtures::*;

/// A dir entry with a named capture and no id reports E026.
#[test]
fn dir_entry_with_named_capture_and_no_id_is_e026() {
    // Arrange
    let entry = YamlEntry {
        id: None,
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
    assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
    let SemanticError::CaptureWithoutId { captures, .. } = &errors[0] else {
        panic!("expected CaptureWithoutId, got {:?}", errors[0]);
    };
    assert_eq!(captures, &["stem".to_string()]);
}
