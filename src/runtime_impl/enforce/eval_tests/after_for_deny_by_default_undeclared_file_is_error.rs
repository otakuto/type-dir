use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_literal, tree_with_files,
};

/// Undeclared files produce an Undeclared error (deny-by-default works on entries after for expansion).
#[test]
fn after_for_deny_by_default_undeclared_file_is_error() {
    // Arrange: for {id: x, value: ["a"]} { file: '${value.x}.sql' } and the node has both a.sql and extra.sql
    let file_entry = make_file_entry(ExprPattern::Exact("${value.x}.sql".to_string()), None);
    let for_entry = make_for_entry_literal("x", vec!["a"], vec![file_entry]);
    let tree = tree_with_files("root", vec!["a.sql", "extra.sql"]);
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

    // Assert: extra.sql is undeclared, so 1 Undeclared error
    assert_eq!(errors.len(), 1);
    let LintError::Undeclared { path: err_path, .. } = &errors[0] else {
        panic!("expected Undeclared, got: {:?}", errors[0]);
    };
    assert!(err_path.ends_with("extra.sql"), "path: {:?}", err_path);
}
