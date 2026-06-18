use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{
    ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprRule, ExprSubtree, Quant,
};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::walk::DirTree;
use crate::yaml::{EntryId, RuleName, VarName};

/// A For entry with an id inside a Splice bubbles its produced records up to the root level.
#[test]
fn produced_for_id_inside_splice_bubbles_up() {
    // Arrange: root tree with files "a.txt" and "b.txt"
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec!["a.txt".to_string(), "b.txt".to_string()],
    };

    // file entry matching "${item}.txt" with id "f"
    let file_entry = ExprEntry {
        id: Some(EntryId("f".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("${value.item}.txt".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    // for entry: for item in ["a", "b"] { file: "${item}.txt" with id "f" }, with id "loop"
    let for_entry = ExprEntry {
        id: Some(EntryId("loop".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("item".to_string()),
            source: ExprForSource::Literal(vec!["a".to_string(), "b".to_string()]),
            body: vec![file_entry],
        },
    };

    // Build the "mod" rule whose body contains the for entry
    let mod_rule = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![for_entry],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("mod".to_string()), mod_rule);

    // Root entries: a single Use entry of "mod"
    let splice_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Use {
            rule: RuleName("mod".to_string()),
            with_args: IndexMap::new(),
        },
    };
    let entries = vec![splice_entry];
    let scope = empty_scope();
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

    // Assert: produced["loop"] has 2 Records (one per binding), and body id "f" does not leak
    let loop_records = produced.get("loop").expect("expected produced[\"loop\"]");
    assert_eq!(
        loop_records.len(),
        2,
        "expected 2 records in produced[\"loop\"] (one per binding), got: {:?}",
        loop_records
    );
    assert!(
        !produced.contains_key("f"),
        "body id \"f\" must not leak to top-level produced: {:?}",
        produced.keys().collect::<Vec<_>>()
    );
}
