use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::walk::DirTree;

#[test]
fn test_check_false_entry_skips_enforcement() {
    // Arrange
    let subtree = DirTree {
        name: "frontend".to_string(),
        dirs: vec![DirTree {
            name: "anything_goes".to_string(),
            dirs: vec![],
            files: vec![],
        }],
        files: vec!["also_fine.ts".to_string()],
    };
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![subtree],
        files: vec![],
    };
    let entries = vec![ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("frontend".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    }];
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
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
