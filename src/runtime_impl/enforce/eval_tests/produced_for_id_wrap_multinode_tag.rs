use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::record_map::RecordMap;
use crate::runtime_impl::value::Record;
use crate::walk::DirTree;
use crate::yaml::{EntryId, VarName};

/// Multi-node one_of in a for body: the winner alternative's id becomes the tag of the for-wrap record.
///
/// Tree: root/ with dirs a/, b/ and file c.txt.
/// `produced["dirs"]` has 3 records for a, b, c (simulating a fetch).
/// for x in ${dirs} / id: classified
///   one_of / id: kind
///     id: pair  :: [dir `${x.regex.n}` :: [file .keep]]
///     id: single :: [file `${x.regex.n}.txt`]
///
/// Expected: produced["classified"] has 3 records with tags pair/pair/single.
#[test]
fn produced_for_id_wrap_multinode_tag() {
    // Arrange
    let tree = DirTree {
        name: ".".to_string(),
        dirs: vec![
            DirTree {
                name: "a".to_string(),
                dirs: vec![],
                files: vec![".keep".to_string()],
            },
            DirTree {
                name: "b".to_string(),
                dirs: vec![],
                files: vec![".keep".to_string()],
            },
        ],
        files: vec!["c.txt".to_string()],
    };

    let make_rec = |letter: &str| Record {
        fields: [
            ("0".to_string(), letter.to_string()),
            ("n".to_string(), letter.to_string()),
        ]
        .into_iter()
        .collect(),
        children: IndexMap::new(),
        tag: None,
    };

    // pair alt: dir `${x.regex.n}/` with child .keep
    let keep_file = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact(".keep".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    let pair_dir = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("${value.x.regex.n}".to_string()),
            subtree: ExprSubtree::Inline(vec![keep_file]),
        },
    };
    let pair_alt = ExprEntry {
        id: Some(EntryId("pair".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![pair_dir],
        },
    };

    // single alt: file `${x.regex.n}.txt`
    let single_file = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("${value.x.regex.n}.txt".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    let single_alt = ExprEntry {
        id: Some(EntryId("single".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Group {
            subtree: vec![single_file],
        },
    };

    // one_of / id: kind
    let one_of_entry = ExprEntry {
        id: Some(EntryId("kind".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![pair_alt, single_alt],
        },
    };

    // for x in ${dirs} / id: classified
    let for_entry = ExprEntry {
        id: Some(EntryId("classified".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("x".to_string()),
            source: ExprForSource::Expr("${dir.dirs}".to_string()),
            body: vec![one_of_entry],
        },
    };

    let mut scope = empty_scope();
    // Place the record set producer on the env side (Dir) (referenced via `${dir.dirs}`).
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "dirs",
        vec![make_rec("a"), make_rec("b"), make_rec("c")],
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

    // Assert: produced["classified"] has 3 records with tags pair/pair/single
    let classified = produced
        .get("classified")
        .expect("expected produced[\"classified\"]");
    assert_eq!(
        classified.len(),
        3,
        "expected 3 wrap records (one per binding), got: {:?}",
        classified
    );
    assert_eq!(
        classified[0].tag.as_deref(),
        Some("pair"),
        "binding 'a' (dir exists) should get tag=pair, got: {:?}",
        classified[0].tag
    );
    assert_eq!(
        classified[1].tag.as_deref(),
        Some("pair"),
        "binding 'b' (dir exists) should get tag=pair, got: {:?}",
        classified[1].tag
    );
    assert_eq!(
        classified[2].tag.as_deref(),
        Some("single"),
        "binding 'c' (only file) should get tag=single, got: {:?}",
        classified[2].tag
    );
}
