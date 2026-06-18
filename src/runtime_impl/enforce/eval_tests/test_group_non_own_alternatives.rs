use std::path::Path;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_literal,
};
use crate::walk::DirTree;

/// Helper to construct a `one_of` alternative.
fn one_of(alternatives: Vec<ExprEntry>) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: alternatives,
        },
    }
}

fn file_entry(name: &str) -> ExprEntry {
    make_file_entry(ExprPattern::Exact(name.to_string()), None)
}

/// Nested group: a one_of whose alternative is itself a one_of (non-Own) coexisting with a sibling file.
///
/// Regression test for the case where a nested Group was passed to eval_consume (Own-only) and
/// caused an unreachable! panic. Only a.txt exists; inner one_of realizes (1) and outer one_of
/// also realizes (1), so no errors.
#[test]
fn test_nested_group_realizes_no_error() {
    // Arrange: only a.txt exists.
    let tree = DirTree {
        name: "n".to_string(),
        dirs: vec![],
        files: vec!["a.txt".to_string()],
    };
    let inner = one_of(vec![file_entry("a.txt"), file_entry("b.txt")]);
    let entries = vec![one_of(vec![inner, file_entry("c.txt")])];
    let scope = empty_scope();
    let path = Path::new("n");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &Default::default(),
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: no panic and exactly 1 realization, so no errors.
    assert!(
        errors.is_empty(),
        "expected no errors for nested group with one realized alternative: {:?}",
        errors
    );
}

/// Nested group: the inner one_of holds both a.txt and b.txt, fails to realize due to cardinality
/// violation, and c.txt (the sibling) is also absent, so the outer one_of has 0 realizations and
/// reports a violation.
#[test]
fn test_nested_group_violation_reported() {
    // Arrange: both a.txt and b.txt exist (inner one_of exceeds max=1 and does not realize).
    let tree = DirTree {
        name: "n".to_string(),
        dirs: vec![],
        files: vec!["a.txt".to_string(), "b.txt".to_string()],
    };
    let inner = one_of(vec![file_entry("a.txt"), file_entry("b.txt")]);
    let entries = vec![one_of(vec![inner, file_entry("c.txt")])];
    let scope = empty_scope();
    let path = Path::new("n");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &Default::default(),
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: no panic, and at least one error is reported (outer one_of has 0 realizations).
    assert!(
        !errors.is_empty(),
        "expected an error when no nested-group alternative realizes"
    );
}

/// For-in-group: a one_of whose alternative is a for block (non-Own) coexisting with a sibling file.
///
/// Verifies that the content model after for binding expansion is correctly realization-checked
/// without panicking. Both a.txt and b.txt exist and the for block realizes (1), so no errors.
#[test]
fn test_for_in_group_realizes_no_error() {
    // Arrange: both a.txt and b.txt exist.
    let tree = DirTree {
        name: "n".to_string(),
        dirs: vec![],
        files: vec!["a.txt".to_string(), "b.txt".to_string()],
    };
    let for_alt = make_for_entry_literal(
        "x",
        vec!["a", "b"],
        vec![make_file_entry(
            ExprPattern::Exact("${value.x}.txt".to_string()),
            None,
        )],
    );
    let entries = vec![one_of(vec![for_alt, file_entry("c.txt")])];
    let scope = empty_scope();
    let path = Path::new("n");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &Default::default(),
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: no panic and the for block realizes, so no errors.
    assert!(
        errors.is_empty(),
        "expected no errors for for-in-group with a realized for block: {:?}",
        errors
    );
}
