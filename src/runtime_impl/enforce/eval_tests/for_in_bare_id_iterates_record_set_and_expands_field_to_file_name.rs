use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_expr, tree_with_files,
};
use crate::runtime_impl::value::Record;

/// Iterates a record set from GlobalSets via bare `${id}` and expands fields into file names.
#[test]
fn for_in_bare_id_iterates_record_set_and_expands_field_to_file_name() {
    // Arrange: place 2 records in sets["a"]: fields{name:"foo"} and fields{name:"bar"}.
    // for {id: x, value: ${a}} { file: '${value.x.regex.name}.rs' } requires foo.rs and bar.rs.
    let file_entry = make_file_entry(
        ExprPattern::Exact("${value.x.regex.name}.rs".to_string()),
        None,
    );
    let for_entry = make_for_entry_expr("x", "${a}", vec![file_entry]);
    let tree = tree_with_files("root", vec!["foo.rs", "bar.rs"]);
    let entries = vec![for_entry];
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    let mut rec_foo = Record::default();
    rec_foo.fields.insert("name".to_string(), "foo".to_string());
    let mut rec_bar = Record::default();
    rec_bar.fields.insert("name".to_string(), "bar".to_string());
    let mut scope = empty_scope();
    // Place the external record set producer on the env side (For). The bare `${a}` resolves via transparent get.
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::For,
        "a",
        vec![rec_foo, rec_bar],
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

    // Assert: foo.rs and bar.rs both exist, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
