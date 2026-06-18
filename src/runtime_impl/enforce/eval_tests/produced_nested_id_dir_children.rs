use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::walk::DirTree;
use crate::yaml::EntryId;

/// An id-bearing dir entry wraps its children: produced["a"][0].children["b"] holds the
/// child file record when the child file entry also has an id.
#[test]
fn produced_nested_id_dir_children() {
    // Arrange: tree root/src/main.rs
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![DirTree {
            name: "src".to_string(),
            dirs: vec![],
            files: vec!["main.rs".to_string()],
        }],
        files: vec![],
    };

    // inner file entry with id "b"
    let file_entry = ExprEntry {
        id: Some(EntryId("b".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("main.rs".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    // dir entry "src" with id "a", containing the file entry
    let dir_entry = ExprEntry {
        id: Some(EntryId("a".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("src".to_string()),
            subtree: ExprSubtree::Inline(vec![file_entry]),
        },
    };
    let entries = vec![dir_entry];
    let scope = empty_scope();
    let rules = IndexMap::new();
    let path = Path::new("root");

    // Act
    let mut errors = Vec::new();
    let mut produced = RecordMap::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut produced,
        &mut TrialMemo::new(),
    );

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let a_records = produced.get("a").expect("expected produced[\"a\"]");
    assert_eq!(
        a_records.len(),
        1,
        "expected exactly 1 record in produced[\"a\"]"
    );
    let a_record = &a_records[0];
    let b_children = a_record
        .children
        .get("b")
        .expect("expected children[\"b\"] in produced[\"a\"][0]");
    assert_eq!(
        b_children.len(),
        1,
        "expected exactly 1 child record under \"b\""
    );
}
