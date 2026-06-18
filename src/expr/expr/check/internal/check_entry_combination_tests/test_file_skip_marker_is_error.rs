use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Verifies that `file: foo/*` is invalid because the `/*` marker cannot be used on a file entry.
#[test]
fn file_skip_marker_is_error() {
    // Arrange: a file entry with the `/*` marker attached.
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("foo/*".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert: an InvalidPattern error is raised.
    assert_eq!(errors.len(), 1, "expected 1 error, got: {errors:?}");
    assert!(
        matches!(&errors[0], SemanticError::InvalidPattern { .. }),
        "expected InvalidPattern, got: {:?}",
        errors[0]
    );
}
