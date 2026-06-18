use std::path::Path;

use indexmap::IndexMap;

use crate::expr::ExprRule;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{docs_dir, empty_tree, make_for_layer_entries};
use crate::runtime_impl::env::Scope;
use crate::walk::DirTree;
use crate::yaml::RuleName;

/// Proof of self-binding isolation: placing the foreign x.txt (from another binding) in bbb-docs results in Undeclared.
/// When bindings are isolated, bbb's m is {y} only, so x.txt is rejected as an undeclared name
/// (if merged, m={x,y} and x.txt would be accepted on the bbb side as well).
#[test]
fn test_for_binding_scope_foreign_is_undeclared() {
    // Arrange: place the foreign x.txt in bbb-docs (y.txt is absent)
    let entries = make_for_layer_entries();
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![
            empty_tree("aaa-pkg-x"),
            docs_dir("aaa-docs", &["x.txt"]),
            empty_tree("bbb-pkg-y"),
            docs_dir("bbb-docs", &["x.txt"]),
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

    // Assert: bbb-docs/x.txt is Undeclared, and MissingRequired fires for bbb-docs/y.txt
    assert!(!errors.is_empty(), "expected binding isolation errors");
    let has_foreign_undeclared = errors.iter().any(|e| match e {
        crate::error::LintError::Undeclared { path, .. } => {
            path.to_string_lossy().contains("bbb-docs/x.txt")
        }
        _ => false,
    });
    assert!(
        has_foreign_undeclared,
        "expected Undeclared for bbb-docs/x.txt: {:?}",
        errors
    );
}
