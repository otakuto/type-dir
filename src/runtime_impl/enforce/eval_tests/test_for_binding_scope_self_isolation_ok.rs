use std::path::Path;

use indexmap::IndexMap;

use crate::expr::ExprRule;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{docs_dir, empty_tree, make_for_layer_entries};
use crate::runtime_impl::env::Scope;
use crate::walk::DirTree;
use crate::yaml::RuleName;

/// Self-binding isolation: m collected per layer binding is visible only within that binding's docs.
/// aaa-pkg-x → aaa-docs/x.txt, bbb-pkg-y → bbb-docs/y.txt are consistent — no errors.
#[test]
fn test_for_binding_scope_self_isolation_ok() {
    // Arrange
    let entries = make_for_layer_entries();
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![
            empty_tree("aaa-pkg-x"),
            docs_dir("aaa-docs", &["x.txt"]),
            empty_tree("bbb-pkg-y"),
            docs_dir("bbb-docs", &["y.txt"]),
        ],
        files: vec![],
    };
    let scope = Scope::new();
    let rules: IndexMap<RuleName, ExprRule> = IndexMap::new();
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
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
