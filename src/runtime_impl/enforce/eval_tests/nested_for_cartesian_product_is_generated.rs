use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_literal, tree_with_files,
};

/// Nested `for` loops generate a Cartesian product.
#[test]
fn nested_for_cartesian_product_is_generated() {
    // Arrange: for {id: x, value: ["foo","bar"]} { for {id: y, value: ["create","list"]} { file: '${value.x}_${value.y}.rs' } }
    // → 4 files required: foo_create.rs, foo_list.rs, bar_create.rs, bar_list.rs
    let file_entry = make_file_entry(
        ExprPattern::Exact("${value.x}_${value.y}.rs".to_string()),
        None,
    );
    let inner_for = make_for_entry_literal("y", vec!["create", "list"], vec![file_entry]);
    let outer_for = make_for_entry_literal("x", vec!["foo", "bar"], vec![inner_for]);
    let tree = tree_with_files(
        "root",
        vec![
            "foo_create.rs",
            "foo_list.rs",
            "bar_create.rs",
            "bar_list.rs",
        ],
    );
    let entries = vec![outer_for];
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

    // Assert: all 4 files exist, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
