use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry};
use crate::walk::DirTree;

/// one_of (optional: min=0, max=1): both alternatives exist → CardinalityViolation
#[test]
fn test_oneof_optional_both_match_violation() {
    // Arrange
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["a.rs".to_string(), "b.rs".to_string()],
    };
    let alt1 = make_file_entry(ExprPattern::Exact("a.rs".to_string()), None);
    let alt2 = make_file_entry(ExprPattern::Exact("b.rs".to_string()), None);
    let group_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 0,
            max: Some(1),
            body: vec![alt1, alt2],
        },
    };
    let entries = vec![group_entry];
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
        "expected CardinalityViolation for optional both-exist: {:?}",
        errors
    );
    let LintError::CardinalityViolation {
        realized, min, max, ..
    } = violations[0]
    else {
        panic!("expected CardinalityViolation");
    };
    assert_eq!(*realized, 2);
    assert_eq!(*min, 0);
    assert_eq!(*max, Some(1));
}
