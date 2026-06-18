use std::path::Path;

use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    content_choice_rules, empty_scope, make_splice_group,
};
use crate::walk::DirTree;

/// any_of: A fails, B succeeds → valid=1 → no errors.
#[test]
fn test_content_choice_any_of_one_valid_no_error() {
    // Arrange: only group.toml exists (resource_dir fails, resource_group_dir succeeds)
    let rules = content_choice_rules();
    let tree = DirTree {
        name: "foo".to_string(),
        dirs: vec![],
        files: vec!["group.toml".to_string()],
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

    // Assert: 1 valid, so no errors
    assert!(
        errors.is_empty(),
        "expected no errors when 1 is valid: {:?}",
        errors
    );
}
