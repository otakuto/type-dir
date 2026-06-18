use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry, tree_with_files};
use crate::runtime_impl::value::Record;

/// Expands a record variable field in scope to a file name via `${x.field}` (through scope binding).
///
/// Sets x=Value::Record(fields{name:"foo"}) directly in scope and verifies that
/// `${x.regex.name}_handler.rs` is generated.
#[test]
fn record_in_scope_expands_field_to_file_name() {
    // Arrange: place x=Record{name:"foo"} in scope and verify that
    // file: '${x.regex.name}_handler.rs' matches foo_handler.rs.
    let file_entry = make_file_entry(
        ExprPattern::Exact("${x.regex.name}_handler.rs".to_string()),
        None,
    );
    let tree = tree_with_files("root", vec!["foo_handler.rs"]);
    let entries = vec![file_entry];
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    let mut scope = empty_scope();
    let mut rec = Record::default();
    rec.fields.insert("name".to_string(), "foo".to_string());
    // Place the record binding on the lex side (Value). The bare `${x.regex.name}` resolves via transparent get.
    scope.bind_lex(
        crate::runtime_impl::node_id::NodeKind::Value,
        "x",
        crate::runtime_impl::value::Value::Record(rec),
    );

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

    // Assert: foo_handler.rs exists, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
