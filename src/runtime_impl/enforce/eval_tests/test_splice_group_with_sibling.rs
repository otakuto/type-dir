use std::path::Path;

use crate::expr::ExprPattern;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    content_choice_rules, empty_scope, make_file_entry, make_splice_group,
};
use crate::walk::DirTree;

/// When a sibling entry (file) coexists with a splice alternative in a one_of, the single-group
/// early return for content-choice mode is not taken; instead each alternative is realization-checked
/// via the multinode group path.
///
/// Regression test for the case where a Splice alternative was passed to eval_consume and caused
/// an unreachable! panic. Only res.toml exists with sibling.txt; only resource_dir realizes and
/// satisfies the one_of.
#[test]
fn test_splice_group_with_sibling_one_valid_no_error() {
    // Arrange: sibling.txt (declared) and res.toml (required by resource_dir) both exist.
    let rules = content_choice_rules();
    let tree = DirTree {
        name: "foo".to_string(),
        dirs: vec![],
        files: vec!["sibling.txt".to_string(), "res.toml".to_string()],
    };
    let entries = vec![
        make_file_entry(ExprPattern::Exact("sibling.txt".to_string()), None),
        make_splice_group(
            1,
            Some(1), // one_of
            &["resource_dir", "resource_group_dir"],
        ),
    ];
    let scope = empty_scope();
    let path = Path::new("envs/foo");

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

    // Assert: no panic and exactly 1 alternative realizes, so no errors.
    assert!(
        errors.is_empty(),
        "expected no errors when exactly 1 splice alternative realizes with a sibling: {:?}",
        errors
    );
}

/// When both splice alternatives realize in a one_of with a sibling, a cardinality violation is reported.
#[test]
fn test_splice_group_with_sibling_both_valid_violation() {
    // Arrange: both res.toml and group.toml exist, so both alternatives realize.
    let rules = content_choice_rules();
    let tree = DirTree {
        name: "foo".to_string(),
        dirs: vec![],
        files: vec![
            "sibling.txt".to_string(),
            "res.toml".to_string(),
            "group.toml".to_string(),
        ],
    };
    let entries = vec![
        make_file_entry(ExprPattern::Exact("sibling.txt".to_string()), None),
        make_splice_group(1, Some(1), &["resource_dir", "resource_group_dir"]),
    ];
    let scope = empty_scope();
    let path = Path::new("envs/foo");

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

    // Assert: 2 realizations exceed one_of (max=1), so a cardinality violation is reported.
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, crate::error::LintError::CardinalityViolation { .. })),
        "expected a cardinality violation when both splice alternatives realize: {:?}",
        errors
    );
}
