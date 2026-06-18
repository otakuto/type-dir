use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_expr, tree_with_files,
};
use crate::runtime_impl::value::Value;

/// `in: ${var}` iterates over a Set variable in scope (binds each element).
#[test]
fn for_in_scope_set_var_iterates_each_element() {
    // Arrange: scope has domains=Set(["entity","usecase"]) → each dir is required
    let file_entry = make_file_entry(ExprPattern::Exact("${value.x}.rs".to_string()), None);
    let for_entry = make_for_entry_expr("x", "${domains}", vec![file_entry]);
    let tree = tree_with_files("root", vec!["entity.rs", "usecase.rs"]);
    let entries = vec![for_entry];
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    let mut scope = empty_scope();
    // Place the Set binding on the lex side (Value). The bare `${domains}` resolves via transparent get.
    scope.bind_lex(
        crate::runtime_impl::node_id::NodeKind::Value,
        "domains",
        Value::Set(vec!["entity".to_string(), "usecase".to_string()]),
    );

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

    // Assert: both files exist, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
