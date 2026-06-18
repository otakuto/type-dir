use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry, make_splice_group};
use crate::walk::DirTree;
use crate::yaml::{RegexPattern, RuleName};

/// one_of: both valid (valid=2) → reports CardinalityViolation as ambiguous.
#[test]
fn test_content_choice_one_of_both_valid_violation() {
    // Arrange: both res.toml and group.toml exist → both rules are valid.
    // However, since each rule marks undeclared files as Undeclared, the tree has both files
    // and each rule accepts the other's file via a regex to create the ambiguous case.
    let resource_dir_rule = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![
            make_file_entry(ExprPattern::Exact("res.toml".to_string()), None),
            // Allow extra files (to create the ambiguous case)
            make_file_entry(
                ExprPattern::Regex(RegexPattern(r"^.*$".to_string())),
                Some((0, Some(1))),
            ),
        ],
    };
    let resource_group_dir_rule = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![
            make_file_entry(ExprPattern::Exact("group.toml".to_string()), None),
            make_file_entry(
                ExprPattern::Regex(RegexPattern(r"^.*$".to_string())),
                Some((0, Some(1))),
            ),
        ],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("resource_dir".to_string()), resource_dir_rule);
    rules.insert(
        RuleName("resource_group_dir".to_string()),
        resource_group_dir_rule,
    );

    let tree = DirTree {
        name: "foo".to_string(),
        dirs: vec![],
        files: vec!["res.toml".to_string(), "group.toml".to_string()],
    };
    let entries = vec![make_splice_group(
        1,
        Some(1), // one_of
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

    // Assert: valid=2 exceeds one_of max=1, so CardinalityViolation
    assert_eq!(
        errors.len(),
        1,
        "expected 1 CardinalityViolation for ambiguity: {:?}",
        errors
    );
    let LintError::CardinalityViolation {
        realized, min, max, ..
    } = &errors[0]
    else {
        panic!("expected CardinalityViolation: {:?}", errors[0]);
    };
    assert_eq!(*realized, 2);
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(1));
}
