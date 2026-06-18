use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::runtime_impl::value::Record;
use crate::walk::DirTree;
use crate::yaml::{EntryId, VarName};

/// For-wrap record carries the binding record's fields when the for source is a record set.
///
/// Setup: `produced["dirs"]` contains 2 records with field `n`: "a" and "b".
/// `for x in ${dirs} / id: classified` with empty body.
/// Expected: `produced["classified"]` has 2 wrap records, each carrying `fields["n"]`.
#[test]
fn produced_for_id_wrap_carries_binding_fields() {
    // Arrange
    let tree = DirTree {
        name: ".".to_string(),
        dirs: vec![],
        files: vec![],
    };

    let make_rec = |n: &str| Record {
        fields: [
            ("0".to_string(), n.to_string()),
            ("n".to_string(), n.to_string()),
        ]
        .into_iter()
        .collect(),
        children: IndexMap::new(),
        tag: None,
    };

    let for_entry = ExprEntry {
        id: Some(EntryId("classified".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("x".to_string()),
            source: ExprForSource::Expr("${dir.dirs}".to_string()),
            body: vec![],
        },
    };

    let mut scope = empty_scope();
    // Place the record set producer on the env side (Dir) (referenced via `${dir.dirs}`).
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "dirs",
        vec![make_rec("a"), make_rec("b")],
    );
    let rules = IndexMap::new();
    let path = Path::new(".");

    let mut errors = Vec::new();
    let mut produced = RecordMap::new();

    // Act
    eval_node(
        &tree,
        &[for_entry],
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut produced,
        &mut TrialMemo::new(),
    );

    // Assert: no errors, produced["classified"] has 2 wrap records carrying binding fields
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let classified = produced
        .get("classified")
        .expect("expected produced[\"classified\"]");
    assert_eq!(
        classified.len(),
        2,
        "expected 2 wrap records (one per binding), got: {:?}",
        classified
    );
    // Each wrap record must carry the binding record's field "n".
    assert_eq!(
        classified[0].fields.get("n").map(|s| s.as_str()),
        Some("a"),
        "wrap record 0 should carry binding field n=a, got fields: {:?}",
        classified[0].fields
    );
    assert_eq!(
        classified[1].fields.get("n").map(|s| s.as_str()),
        Some("b"),
        "wrap record 1 should carry binding field n=b, got fields: {:?}",
        classified[1].fields
    );
}
