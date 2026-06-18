use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_literal, tree_with_files,
};

/// Files for each element in `in: ["a", "b"]` literal list become required.
#[test]
fn for_in_literal_list_files_become_required() {
    // Arrange: for {id: x, value: ["a","b"]} { file: '${value.x}.sql' } requires a.sql and b.sql in the node
    let file_entry = make_file_entry(ExprPattern::Exact("${value.x}.sql".to_string()), None);
    let for_entry = make_for_entry_literal("x", vec!["a", "b"], vec![file_entry]);
    let tree = tree_with_files("root", vec!["a.sql", "b.sql"]);
    let entries = vec![for_entry];
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &entries,
        &empty_scope(),
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: a.sql and b.sql both exist, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
