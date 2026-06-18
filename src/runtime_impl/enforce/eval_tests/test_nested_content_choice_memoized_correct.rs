use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry, make_splice_group};
use crate::walk::DirTree;
use crate::yaml::RuleName;

/// Even with nested content-choice (an alternative rule's body in one_of is itself another one_of),
/// the result is correct (verifies memoization correctness).
#[test]
fn test_nested_content_choice_memoized_correct() {
    // Arrange:
    // outer one_of [outer_a, outer_b]
    //   outer_a body is one_of [inner_x, inner_y] (another content-choice)
    //     inner_x: file "x.toml" required / inner_y: file "y.toml" required
    //   outer_b: file "b.toml" required
    // tree has only "x.toml" → outer_a is valid via inner_x; outer_b fails.
    // Therefore one_of has exactly 1 valid and produces no errors.
    let inner_x = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_file_entry(
            ExprPattern::Exact("x.toml".to_string()),
            None,
        )],
    };
    let inner_y = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_file_entry(
            ExprPattern::Exact("y.toml".to_string()),
            None,
        )],
    };
    // outer_a body has a single content-choice entry: one_of [inner_x, inner_y].
    let outer_a = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_splice_group(1, Some(1), &["inner_x", "inner_y"])],
    };
    let outer_b = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_file_entry(
            ExprPattern::Exact("b.toml".to_string()),
            None,
        )],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("inner_x".to_string()), inner_x);
    rules.insert(RuleName("inner_y".to_string()), inner_y);
    rules.insert(RuleName("outer_a".to_string()), outer_a);
    rules.insert(RuleName("outer_b".to_string()), outer_b);

    let tree = DirTree {
        name: "foo".to_string(),
        dirs: vec![],
        files: vec!["x.toml".to_string()],
    };
    let entries = vec![make_splice_group(1, Some(1), &["outer_a", "outer_b"])];
    let scope = empty_scope();
    let path = Path::new("envs/foo");

    // Act: evaluate with a shared memo (memoization must not break correctness even when the same trial is re-run in nested calls)
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

    // Assert: only outer_a is valid (matches inner_x) → exactly 1 valid, no errors
    assert!(
        errors.is_empty(),
        "expected no errors when exactly 1 valid in nested content-choice: {:?}",
        errors
    );
}
