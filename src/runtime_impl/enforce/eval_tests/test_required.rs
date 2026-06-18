use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::ExprPattern;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::walk::DirTree;

use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry};

#[test]
fn test_required_file_missing_produces_error() {
    // Arrange
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec![],
    };
    let entries = vec![
        make_file_entry(ExprPattern::Exact("Cargo.toml".to_string()), None),
        make_file_entry(
            ExprPattern::Exact("Cargo.lock".to_string()),
            Some((0, Some(1))),
        ),
    ];
    let scope = empty_scope();
    let rules = IndexMap::new();
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
    let LintError::MissingRequired { parent, name, .. } = &errors[0] else {
        panic!("expected MissingRequired, got {:?}", errors[0]);
    };
    assert_eq!(parent, &PathBuf::from("crate"));
    assert_eq!(name, "Cargo.toml");
}
