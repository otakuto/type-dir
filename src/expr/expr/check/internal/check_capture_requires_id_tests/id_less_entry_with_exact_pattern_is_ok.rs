use indexmap::IndexMap;

use crate::expr::expr::check::internal::check_capture_requires_id::check_capture_requires_id;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};

use super::fixtures::*;

/// An id-less entry with an Exact pattern (no regex at all) is valid (no error).
#[test]
fn id_less_entry_with_exact_pattern_is_ok() {
    // Arrange
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("src".to_string()),
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
