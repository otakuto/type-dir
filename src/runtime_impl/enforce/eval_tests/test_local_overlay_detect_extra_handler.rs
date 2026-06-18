use std::path::{Path, PathBuf};

use crate::error::LintError;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, graphql_node_setup, handler_dir_tree, schema_dir_tree,
};
use crate::walk::DirTree;

/// A handler for an op not present in the local schema is detected as Undeclared.
#[test]
fn test_local_overlay_detect_extra_handler() {
    // Arrange
    let (rules, entries) = graphql_node_setup();
    let tree = DirTree {
        name: "graphql".to_string(),
        dirs: vec![
            handler_dir_tree(
                vec!["a_mutation_handler"],
                vec!["a_mutation_handler.rs", "c_query_handler.rs"],
            ),
            schema_dir_tree(vec!["a_mutation.rs"]),
        ],
        files: vec![],
    };
    let scope = empty_scope();
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

    // Assert: c_query_handler.rs is not in the schema — Undeclared
    assert_eq!(errors.len(), 1, "unexpected errors: {:?}", errors);
    let LintError::Undeclared { path: err_path, .. } = &errors[0] else {
        panic!("expected Undeclared, got {:?}", errors[0]);
    };
    assert_eq!(
        err_path,
        &PathBuf::from("graphql/handler/c_query_handler.rs")
    );
}
