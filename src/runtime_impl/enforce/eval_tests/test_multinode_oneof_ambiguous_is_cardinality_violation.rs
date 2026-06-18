use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry};
use crate::walk::DirTree;

/// one_of(min=1,max=1) where both Record alternatives realize (ambiguous design).
/// Both alternatives declare only file:a.txt, so both realize when a.txt is present.
/// realized=2 > max=1 → CardinalityViolation.
#[test]
fn test_multinode_oneof_ambiguous_is_cardinality_violation() {
    // Arrange
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["a.txt".to_string()],
    };

    let file_entry_1 = make_file_entry(ExprPattern::Exact("a.txt".to_string()), None);
    let alt1 = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![file_entry_1],
        },
    };

    let file_entry_2 = make_file_entry(ExprPattern::Exact("a.txt".to_string()), None);
    let alt2 = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![file_entry_2],
        },
    };

    // one_of: exactly one must realize, but both do (ambiguous)
    let group = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt1, alt2],
        },
    };
    let entries = vec![group];
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

    // Assert
    let violations: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, LintError::CardinalityViolation { .. }))
        .collect();
    assert_eq!(
        violations.len(),
        1,
        "expected exactly one CardinalityViolation when both alternatives realize: {:?}",
        errors
    );
    let LintError::CardinalityViolation {
        realized, min, max, ..
    } = violations[0]
    else {
        panic!("expected CardinalityViolation");
    };
    assert_eq!(*realized, 2, "expected realized=2");
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(1));
}
