use std::path::Path;

use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, graphql_node_setup, handler_dir_tree, schema_dir_tree,
};
use crate::runtime_impl::value::Record;
use crate::walk::DirTree;

/// Even when the global set is contaminated with an op from another instance (b_query),
/// node-local collection (a_mutation only) is used for input resolution and does not leak through.
#[test]
fn test_local_overlay_prevents_scope_leak() {
    // Arrange
    let (rules, entries) = graphql_node_setup();
    let tree = DirTree {
        name: "graphql".to_string(),
        dirs: vec![
            handler_dir_tree(vec!["a_mutation_handler"], vec!["a_mutation_handler.rs"]),
            schema_dir_tree(vec!["a_mutation.rs"]),
        ],
        files: vec![],
    };
    // Pass gql_op = contaminated record list (a_mutation, b_query) from outside into Γ_set of scope.
    // Node-local collection overwrites gql_op with schema-derived records (a_mutation only), so no leakthrough.
    let mut rec_a = Record::default();
    rec_a
        .fields
        .insert("op".to_string(), "a_mutation".to_string());
    let mut rec_b = Record::default();
    rec_b.fields.insert("op".to_string(), "b_query".to_string());
    let mut scope = empty_scope();
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "gql_op",
        vec![rec_a, rec_b],
    );
    let path = Path::new("graphql");

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

    // Assert: local collection constrains op to [a_mutation]; b_query_handler is not required
    assert!(
        errors.is_empty(),
        "scope leakthrough detected: {:?}",
        errors
    );
}
