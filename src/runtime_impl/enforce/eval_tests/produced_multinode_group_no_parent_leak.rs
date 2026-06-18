use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::walk::DirTree;
use crate::yaml::EntryId;

/// A multi-node Group with id "g" wraps realized alternatives; inner ids must NOT appear
/// as top-level keys in produced.
///
/// Tree: root with dir "x" (containing file "keep.txt").
/// Group alternatives:
///   - alt_a (id "alt_a"): Record subtree declaring dir "x" (with inner file id "leaf") and file "extra.txt"
///   - alt_b (id "alt_b"): Record subtree declaring only file "other.txt"
///
/// alt_a does not realize (file "extra.txt" is missing), alt_b does not realize either.
/// Actually: use a simpler tree where exactly alt_a realizes.
///
/// Simpler setup: root with file "data.txt" only.
/// alt_a (id "alt_a"): Record subtree: file "data.txt" with id "leaf"
/// alt_b (id "alt_b"): Record subtree: file "other.txt"
/// one_of(1,1): alt_a realizes; produced["g"] exists; "leaf" and "alt_a" must NOT be top-level.
#[test]
fn produced_multinode_group_no_parent_leak() {
    // Arrange: root with file "data.txt"
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["data.txt".to_string()],
    };

    // alt_a: Record subtree containing file "data.txt" with id "leaf"; alt has id "alt_a"
    let leaf_file = ExprEntry {
        id: Some(EntryId("leaf".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("data.txt".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    let alt_a = ExprEntry {
        id: Some(EntryId("alt_a".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![leaf_file],
        },
    };

    // alt_b: Record subtree containing file "other.txt" (will not realize)
    let other_file = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("other.txt".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    let alt_b = ExprEntry {
        id: Some(EntryId("alt_b".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![other_file],
        },
    };

    // Group with id "g", one_of(1,1)
    let group_entry = ExprEntry {
        id: Some(EntryId("g".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt_a, alt_b],
        },
    };
    let entries = vec![group_entry];
    let scope = empty_scope();
    let rules = IndexMap::new();
    let path = Path::new("root");

    // Act
    let mut errors = Vec::new();
    let mut produced = RecordMap::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut produced,
        &mut TrialMemo::new(),
    );

    // Assert: no errors (alt_a realizes)
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    // produced["g"] must exist
    assert!(
        produced.contains_key("g"),
        "expected produced[\"g\"] to exist"
    );
    // "leaf" must NOT appear as a top-level key
    assert!(
        !produced.contains_key("leaf"),
        "inner id \"leaf\" must not leak to top-level produced: {:?}",
        produced.keys().collect::<Vec<_>>()
    );
    // "alt_a" must NOT appear as a top-level key
    assert!(
        !produced.contains_key("alt_a"),
        "alt id \"alt_a\" must not leak to top-level produced: {:?}",
        produced.keys().collect::<Vec<_>>()
    );
}
