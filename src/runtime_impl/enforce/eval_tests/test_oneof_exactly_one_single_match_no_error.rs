use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry};
use crate::walk::DirTree;

/// one_of (min=1, max=1): only one of the alternatives exists → no errors
#[test]
fn test_oneof_exactly_one_single_match_no_error() {
    // Arrange
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["config.toml".to_string()],
    };
    let alt1 = make_file_entry(ExprPattern::Exact("config.toml".to_string()), None);
    let alt2 = make_file_entry(ExprPattern::Exact("config.yaml".to_string()), None);
    let group_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt1, alt2],
        },
    };
    let entries = vec![group_entry];
    let scope = empty_scope();
    let rules = IndexMap::new();
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
    assert!(
        errors.is_empty(),
        "expected no errors when only one alternative exists: {:?}",
        errors
    );
}
