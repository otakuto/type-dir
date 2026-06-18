use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_dir_entry, make_file_entry};
use crate::walk::DirTree;

/// one_of(min=1,max=1) with two Record alternatives (pair=both, single=exactly-one):
/// when only dir is present, pair does not realize (file missing) and single realizes (inner one_of: dir only=1).
/// realized=1 → no CardinalityViolation. Dir is declared → no Undeclared.
#[test]
fn test_multinode_oneof_single_realizes_when_one_present() {
    // Arrange
    let tree = DirTree {
        name: "items".to_string(),
        dirs: vec![DirTree {
            name: "x".to_string(),
            dirs: vec![],
            files: vec![".keep".to_string()],
        }],
        files: vec![],
    };

    // pair alternative: subtree declares both dir:x and file:x.txt (both required)
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
    let inner_dir = make_dir_entry(
        ExprPattern::Exact("x".to_string()),
        None,
        ExprSubtree::Inline(vec![make_file_entry(
            ExprPattern::Exact(".keep".to_string()),
            None,
        )]),
    );
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

    // Assert
    assert!(
        errors.is_empty(),
        "expected no errors when only dir present (single realizes, pair does not): {:?}",
        errors
    );
}
