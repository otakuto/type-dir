use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprRule, ExprSubtree, Interval, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::splice_entry;
use crate::runtime_impl::env::Scope;
use crate::walk::DirTree;
use crate::yaml::RuleName;

/// A rule is hermetic and cannot reference ambient variables not declared in its with params.
#[test]
fn test_structure_scope_is_hermetic() {
    // Arrange
    let leaf_struct = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![ExprEntry {
            id: None,
            source_path: None,
            count: Quant::Explicit(Interval {
                min: 0,
                max: Some(1),
            }),
            matcher: ExprMatcher::Dir {
                pattern: ExprPattern::Exact("${foo}".to_string()),
                subtree: ExprSubtree::Leaf,
            },
        }],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("leaf_struct".to_string()), leaf_struct);

    // Splice the contents of the wrap dir via leaf_struct
    let wrap_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("wrap".to_string()),
            subtree: ExprSubtree::Inline(vec![splice_entry("leaf_struct", IndexMap::new())]),
        },
    };

    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![DirTree {
            name: "wrap".to_string(),
            dirs: vec![DirTree {
                name: "bar".to_string(),
                dirs: vec![],
                files: vec![],
            }],
            files: vec![],
        }],
        files: vec![],
    };

    // Bind foo=bar in the outer scope (hermetic rule should not see it)
    let mut scope = Scope::new();
    scope.bind_lex(
        crate::runtime_impl::node_id::NodeKind::Regex,
        "foo",
        crate::runtime_impl::value::Value::Scalar("bar".to_string()),
    );
    let path = Path::new("root");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &[wrap_entry],
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: foo does not leak — `${foo}` is not expanded, so bar is Undeclared
    assert_eq!(errors.len(), 1, "unexpected errors: {:?}", errors);
    let LintError::Undeclared { path: err_path, .. } = &errors[0] else {
        panic!("expected Undeclared, got {:?}", errors[0]);
    };
    assert_eq!(err_path, &PathBuf::from("root/wrap/bar"));
}
