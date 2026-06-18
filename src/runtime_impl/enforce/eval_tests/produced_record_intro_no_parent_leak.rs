use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::walk::DirTree;
use crate::yaml::EntryId;

/// A Record-intro entry with id "rec" wraps its subtree; the inner file id "inner" must NOT
/// appear as a top-level key in produced (only "rec" should be present).
#[test]
fn produced_record_intro_no_parent_leak() {
    // Arrange: root with file "data.json"
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["data.json".to_string()],
    };

    // inner file entry with id "inner"
    let file_entry = ExprEntry {
        id: Some(EntryId("inner".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("data.json".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    // Record-intro entry with id "rec" containing the file entry
    let record_entry = ExprEntry {
        id: Some(EntryId("rec".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![file_entry],
        },
    };
    let entries = vec![record_entry];
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

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    // produced["rec"] must exist with children["inner"]
    let rec_records = produced.get("rec").expect("expected produced[\"rec\"]");
    assert_eq!(
        rec_records.len(),
        1,
        "expected exactly 1 record in produced[\"rec\"]"
    );
    assert!(
        rec_records[0].children.contains_key("inner"),
        "expected children[\"inner\"] inside produced[\"rec\"][0]"
    );
    // "inner" must NOT appear as a top-level key (no parent leak)
    assert!(
        !produced.contains_key("inner"),
        "inner id \"inner\" must not leak to top-level produced: {:?}",
        produced.keys().collect::<Vec<_>>()
    );
}
