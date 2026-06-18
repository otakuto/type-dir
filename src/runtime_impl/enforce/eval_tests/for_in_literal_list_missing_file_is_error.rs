use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_literal, tree_with_files,
};

/// When one file is missing from `in: ["a", "b"]`, an error is produced.
#[test]
fn for_in_literal_list_missing_file_is_error() {
    // Arrange: only a.sql exists; b.sql is missing
    let file_entry = make_file_entry(ExprPattern::Exact("${value.x}.sql".to_string()), None);
    let for_entry = make_for_entry_literal("x", vec!["a", "b"], vec![file_entry]);
    let tree = tree_with_files("root", vec!["a.sql"]);
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

    // Assert: b.sql is missing, so 1 error
    assert_eq!(errors.len(), 1);
    let LintError::MissingRequired { name, .. } = &errors[0] else {
        panic!("expected MissingRequired, got: {:?}", errors[0]);
    };
    assert_eq!(name, "b.sql");
}
