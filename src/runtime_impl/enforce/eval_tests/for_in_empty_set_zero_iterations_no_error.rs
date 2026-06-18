use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, empty_tree, make_file_entry, make_for_entry_expr,
};

/// A bare `${id}` reference not found in scope results in zero iterations and no error.
#[test]
fn for_in_empty_set_zero_iterations_no_error() {
    // Arrange: "schema" does not exist in scope → bare `${schema}` iterates 0 times
    let file_entry = make_file_entry(
        ExprPattern::Exact("${value.x}_handler.rs".to_string()),
        None,
    );
    let for_entry = make_for_entry_expr("x", "${schema}", vec![file_entry]);
    let tree = empty_tree("root");
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

    // Assert: set is empty, so no iterations → no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
