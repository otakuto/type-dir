use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprRule, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::value::Record;
use crate::walk::DirTree;
use crate::yaml::VarName;

use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry};

/// In optional record iteration `for r in ${x} { file: '${r.v}.txt' optional }`,
/// a missing element (b.txt) is not an error, but a file outside the set (c.txt) is Undeclared.
///
/// Set iteration is unified into bare `${id}` record iteration (the dotted flat projection `${x.v}`
/// has been removed), so `for r in ${x}` binds each record and `${r.v}` references the field.
/// The required/missing/undeclared semantics are covered by `test_for.rs`/`test_for_record.rs`;
/// this test verifies only the optional semantics not yet covered by for.
#[test]
fn test_enforce_set_optional_missing_ok_but_undeclared_denied() {
    // Arrange
    let tree = DirTree {
        name: "dir".to_string(),
        dirs: vec![],
        files: vec![
            "a.txt".to_string(), // b.txt is absent but that is fine (optional)
            "c.txt".to_string(), // outside the set
        ],
    };

    let mut rec_a = Record::default();
    rec_a.fields.insert("v".to_string(), "a".to_string());
    let mut rec_b = Record::default();
    rec_b.fields.insert("v".to_string(), "b".to_string());

    // for {id: r, value: ${x}} { file: '${value.r.v}.txt' optional }
    let file_entry = make_file_entry(
        ExprPattern::Exact("${value.r.v}.txt".to_string()),
        Some((0, Some(1))),
    );
    let for_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("r".to_string()),
            source: ExprForSource::Expr("${dir.x}".to_string()),
            body: vec![file_entry],
        },
    };

    let entries = vec![for_entry];
    // Set x = record set (v=a, v=b) in Γ_set of scope
    let mut scope = empty_scope();
    // Place the record set producer on the env side (Dir) (referenced via `${dir.x}`).
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "x",
        vec![rec_a, rec_b],
    );
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("dir");

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

    // Assert: missing b.txt is OK (optional); c.txt is Undeclared
    assert_eq!(errors.len(), 1);
    let LintError::Undeclared { path: err_path, .. } = &errors[0] else {
        panic!("expected Undeclared, got {:?}", errors[0]);
    };
    assert_eq!(err_path, &PathBuf::from("dir/c.txt"));
}
