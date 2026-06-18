use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::walk::DirTree;
use crate::yaml::{EntryId, RegexPattern};

/// An id-bearing Group produces a Record with tag = the winning alternative's id.
#[test]
fn produced_group_tag() {
    // Arrange: tree with file "foo.txt"
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["foo.txt".to_string()],
    };

    // Alternative 1: file "foo.txt" with id "txt_alt"
    let alt_txt = ExprEntry {
        id: Some(EntryId("txt_alt".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("foo.txt".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    // Alternative 2: file matching ".md" extension with id "md_alt"
    let alt_md = ExprEntry {
        id: Some(EntryId("md_alt".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Regex(RegexPattern(r"^.+\.md$".to_string())),
            subtree: ExprSubtree::Leaf,
        },
    };
    // Group entry with id "g" (one_of)
    let group_entry = ExprEntry {
        id: Some(EntryId("g".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt_txt, alt_md],
        },
    };
    let entries = vec![group_entry];
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

    // Assert: no errors, produced["g"] has one record with tag = Some("txt_alt")
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let g_records = produced.get("g").expect("expected produced[\"g\"]");
    assert_eq!(
        g_records.len(),
        1,
        "expected exactly 1 record in produced[\"g\"]"
    );
    assert_eq!(
        g_records[0].tag.as_deref(),
        Some("txt_alt"),
        "expected tag = Some(\"txt_alt\") for the winning alternative"
    );
}
