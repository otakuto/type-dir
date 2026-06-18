use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{
    ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprRule, ExprSubtree, Quant,
};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::make_dir_entry;
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::value::Record;
use crate::walk::DirTree;
use crate::yaml::VarName;

/// Cross-tree: iterates a record set from another root via bare `${b}`,
/// requiring each dir as a combination of a lexical scalar (dir_a) × a field of the bound record (right.dir_b).
///
/// Set iteration is unified into bare `${id}` record iteration (the dotted flat projection `${b.dir_b}`
/// has been removed), so `for right in ${b}` binds each record and `${right.dir_b}` references the field.
/// This eval_node unit test pins the "lexical scalar × record set iteration combination" behavior.
#[test]
fn test_cross_tree_pair_dir() {
    // Arrange: A/foo requires foo_x and foo_y — dir_a=foo (lexical) × each record in b with dir_b={x,y}.
    // Set b = record list (dir_b=x, dir_b=y) in scope.
    let mut rec_x = Record::default();
    rec_x.fields.insert("dir_b".to_string(), "x".to_string());
    let mut rec_y = Record::default();
    rec_y.fields.insert("dir_b".to_string(), "y".to_string());

    // for {id: right, value: ${b}} { dir: '${dir_a}_${value.right.dir_b}' }
    let pair_dir = make_dir_entry(
        ExprPattern::Exact("${dir_a}_${value.right.dir_b}".to_string()),
        None,
        ExprSubtree::Leaf,
    );
    let for_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("right".to_string()),
            source: ExprForSource::Expr("${dir.b}".to_string()),
            body: vec![pair_dir],
        },
    };

    // A/foo/ contains foo_x and foo_y
    let foo_dir = DirTree {
        name: "foo".to_string(),
        dirs: vec![
            DirTree {
                name: "foo_x".to_string(),
                dirs: vec![],
                files: vec![],
            },
            DirTree {
                name: "foo_y".to_string(),
                dirs: vec![],
                files: vec![],
            },
        ],
        files: vec![],
    };

    // Set dir_a=foo as a scalar (lexical capture) and b as a record set (id producer equivalent, Γ_set) in scope
    let mut scope = Scope::new();
    // dir_a is a scalar capture (lex-side Regex); b is a record set producer (env-side Dir, referenced via `${dir.b}`).
    scope.bind_lex(
        crate::runtime_impl::node_id::NodeKind::Regex,
        "dir_a",
        crate::runtime_impl::value::Value::Scalar("foo".to_string()),
    );
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "b",
        vec![rec_x, rec_y],
    );

    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("A/foo");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &foo_dir,
        &[for_entry],
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: both foo_x and foo_y exist, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
