use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_dir_entry, make_file_entry};
use crate::walk::DirTree;

/// one_of(min=1,max=1) with two Record alternatives (pair=both, single=exactly-one):
/// when neither dir nor file is present, neither alternative realizes.
/// realized=0 < min=1 → CardinalityViolation.
#[test]
fn test_multinode_oneof_none_realizes_is_violation() {
    // Arrange: empty tree (no dirs, no files)
    let tree = DirTree {
        name: "items".to_string(),
        dirs: vec![],
        files: vec![],
    };

    // pair alternative: both dir:x and file:x.txt required
    let dir_x_entry = make_dir_entry(
        ExprPattern::Exact("x".to_string()),
        None,
        ExprSubtree::Inline(vec![make_file_entry(
            ExprPattern::Exact(".keep".to_string()),
            None,
        )]),
    );
    let file_x_txt_entry = make_file_entry(ExprPattern::Exact("x.txt".to_string()), None);
    let pair_alt = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![dir_x_entry, file_x_txt_entry],
        },
    };

    // single alternative: inner one_of(1,1) requires exactly one of dir:x or file:x.txt
    let inner_dir = make_dir_entry(ExprPattern::Exact("x".to_string()), None, ExprSubtree::Leaf);
    let inner_file = make_file_entry(ExprPattern::Exact("x.txt".to_string()), None);
    let inner_group = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![inner_dir, inner_file],
        },
    };
    let single_alt = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![inner_group],
        },
    };

    // outer one_of: exactly one of pair or single must realize
    let outer_group = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![pair_alt, single_alt],
        },
    };
    let entries = vec![outer_group];
    let scope = empty_scope();
    let rules = IndexMap::new();
    let path = Path::new("items");

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

    // Assert: realized=0 < min=1 → CardinalityViolation expected
    let violations: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, LintError::CardinalityViolation { .. }))
        .collect();
    assert_eq!(
        violations.len(),
        1,
        "expected exactly one CardinalityViolation when neither alternative realizes: {:?}",
        errors
    );
    let LintError::CardinalityViolation {
        realized, min, max, ..
    } = violations[0]
    else {
        panic!("expected CardinalityViolation");
    };
    assert_eq!(*realized, 0, "expected realized=0");
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(1));
}
