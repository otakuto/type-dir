use std::path::Path;

use crate::error::LintError;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, graphql_node_setup, handler_dir_tree, schema_dir_tree,
};
use crate::walk::DirTree;

/// When the handler for a local schema op is missing, MissingRequired is detected.
#[test]
fn test_local_overlay_detect_missing_handler() {
    // Arrange
    let (rules, entries) = graphql_node_setup();
    let tree = DirTree {
        name: "graphql".to_string(),
        dirs: vec![
            handler_dir_tree(vec!["a_mutation_handler"], vec!["a_mutation_handler.rs"]),
            schema_dir_tree(vec!["a_mutation.rs", "b_query.rs"]),
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

    // Assert: b_query_handler (dir) and b_query_handler.rs (file) are both missing
    let missing: Vec<&String> = errors
        .iter()
        .filter_map(|e| match e {
            LintError::MissingRequired { name, .. } => Some(name),
            _ => None,
        })
        .collect();
    assert_eq!(errors.len(), 2, "unexpected errors: {:?}", errors);
    assert!(missing.contains(&&"b_query_handler".to_string()));
    assert!(missing.contains(&&"b_query_handler.rs".to_string()));
}
