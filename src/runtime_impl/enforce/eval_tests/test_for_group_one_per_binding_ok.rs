use std::path::Path;

use indexmap::IndexMap;

use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, for_one_of_entries};
use crate::walk::DirTree;

/// (1) Exactly 1 one_of is realized per for binding (a_1.txt and b_2.txt) → no errors.
#[test]
fn test_for_group_one_per_binding_ok() {
    // Arrange
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["a_1.txt".to_string(), "b_2.txt".to_string()],
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

    // Assert
    assert!(
        errors.is_empty(),
        "expected no errors when one_of is realized exactly once per binding: {:?}",
        errors
    );
}
