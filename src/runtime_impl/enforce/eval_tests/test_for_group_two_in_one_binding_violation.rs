use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, for_one_of_entries};
use crate::walk::DirTree;

/// (2) The a binding's one_of is realized twice (both a_1.txt and a_2.txt exist)
/// → a binding's one_of exceeds max=1, resulting in CardinalityViolation.
#[test]
fn test_for_group_two_in_one_binding_violation() {
    // Arrange
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["a_1.txt".to_string(), "a_2.txt".to_string()],
    };
    let entries = for_one_of_entries();
    let scope = empty_scope();
    let rules = IndexMap::new();
    let path = Path::new("root");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: a binding's one_of realized=2 exceeds max=1, CardinalityViolation exists
    let over_max = errors.iter().any(|e| {
        matches!(
            e,
            LintError::CardinalityViolation {
                realized, max: Some(m), ..
            } if realized > m
        )
    });
    assert!(
        over_max,
        "expected CardinalityViolation for a binding's one_of realized=2: {:?}",
        errors
    );
}
