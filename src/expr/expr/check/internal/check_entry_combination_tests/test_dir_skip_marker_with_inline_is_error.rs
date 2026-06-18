use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

/// Verifies that combining `dir: foo/*` with an inline `::` block is invalid.
#[test]
fn dir_skip_marker_with_inline_is_error() {
    // Arrange: a dir entry that has both the `/*` marker and an inline `::` block.
    let child = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("bar.txt".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("foo/*".to_string()),
            body: Some(vec![child]),
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
