use std::path::Path;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{
    empty_scope, make_file_entry, make_for_entry_expr, tree_with_files,
};
use crate::runtime_impl::value::Record;

/// Nested for iterates a child set and preserves the grouping of outer records.
///
/// sets["a"] = [foo(children["b"]=[foo1,foo2]), bar(children["b"]=[bar1])]
/// for x in ${a} { for y in ${x.dir.b} { file: '${x.regex.name}_${y.regex.file}.rs' } }
/// 3 files are generated: foo×foo1, foo×foo2, bar×bar1 (bar1 is not mixed into foo's y).
#[test]
fn for_nested_child_set_iteration_preserves_group() {
    // Arrange
    let file_entry = make_file_entry(
        ExprPattern::Exact("${value.x.regex.name}_${value.y.regex.file}.rs".to_string()),
        None,
    );
    let inner_for = make_for_entry_expr("y", "${value.x.dir.b}", vec![file_entry]);
    let outer_for = make_for_entry_expr("x", "${a}", vec![inner_for]);
    let tree = tree_with_files("root", vec!["foo_foo1.rs", "foo_foo2.rs", "bar_bar1.rs"]);
    let entries = vec![outer_for];
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    // foo record: children["b"] = [fields{file:"foo1"}, fields{file:"foo2"}]
    let mut rec_foo1 = Record::default();
    rec_foo1
        .fields
        .insert("file".to_string(), "foo1".to_string());
    let mut rec_foo2 = Record::default();
    rec_foo2
        .fields
        .insert("file".to_string(), "foo2".to_string());
    let mut rec_foo = Record::default();
    rec_foo.fields.insert("name".to_string(), "foo".to_string());
    rec_foo.children.insert(
        "b".to_string(),
        vec![Arc::new(rec_foo1), Arc::new(rec_foo2)],
    );

    // bar record: children["b"] = [fields{file:"bar1"}]
    let mut rec_bar1 = Record::default();
    rec_bar1
        .fields
        .insert("file".to_string(), "bar1".to_string());
    let mut rec_bar = Record::default();
    rec_bar.fields.insert("name".to_string(), "bar".to_string());
    rec_bar
        .children
        .insert("b".to_string(), vec![Arc::new(rec_bar1)]);

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

    // Assert: all 3 files exist and grouping is preserved, so no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
