use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprPattern, ExprRule, ExprSubtree};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_dir_entry, make_file_entry, splice_entry,
};
use crate::walk::DirTree;
use crate::yaml::{RegexPattern, RuleName};

#[test]
fn test_contents_only_structure_recursion() {
    // Arrange: src_tree is a content-model rule (only *.rs files are allowed)
    let src_tree_structure = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_file_entry(
            ExprPattern::Regex(RegexPattern(r"^[a-z][a-z0-9_]*\.rs$".to_string())),
            Some((0, Some(1))),
        )],
    };
    let rule_name = RuleName("src_tree".to_string());
    let mut rules = IndexMap::new();
    rules.insert(rule_name.clone(), src_tree_structure);

    let inner_src = DirTree {
        name: "src".to_string(),
        dirs: vec![],
        files: vec!["lib.rs".to_string(), "INVALID_NAME.rs".to_string()],
    };
    let tree = DirTree {
        name: "crate".to_string(),
        dirs: vec![inner_src],
        files: vec![],
    };

    // Splice the contents of the src dir via src_tree
    let entries = vec![make_dir_entry(
        ExprPattern::Exact("src".to_string()),
        None,
        ExprSubtree::Inline(vec![splice_entry("src_tree", IndexMap::new())]),
    )];
    let scope = empty_scope();
    let path = Path::new("crate");

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
    assert_eq!(errors.len(), 1);
    let LintError::Undeclared { path: err_path, .. } = &errors[0] else {
        panic!("expected Undeclared, got {:?}", errors[0]);
    };
    assert_eq!(err_path, &PathBuf::from("crate/src/INVALID_NAME.rs"));
}
