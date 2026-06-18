use std::path::Path;

use crate::error::LintError;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    content_choice_rules, empty_scope, make_splice_group,
};
use crate::walk::DirTree;

/// any_of: both fail (valid=0) → reports the error from the closest branch.
#[test]
fn test_content_choice_any_of_none_valid_reports_closest() {
    // Arrange: no required files exist (both fail; each trial has the same error count)
    let rules = content_choice_rules();
    let tree = DirTree {
        name: "foo".to_string(),
        dirs: vec![],
        files: vec![],
    };
    let entries = vec![make_splice_group(
        1,
        None, // any_of
        &["resource_dir", "resource_group_dir"],
    )];
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

    // Assert: valid=0, so the closest branch (first = resource_dir) error is reported
    assert_eq!(
        errors.len(),
        1,
        "expected 1 error from the closest branch: {:?}",
        errors
    );
    let LintError::MissingRequired { name, .. } = &errors[0] else {
        panic!("expected MissingRequired: {:?}", errors[0]);
    };
    assert_eq!(name, "res.toml");
}
