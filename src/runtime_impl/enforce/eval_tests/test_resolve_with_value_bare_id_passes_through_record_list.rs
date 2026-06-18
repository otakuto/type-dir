use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::enforce::with::resolve_with_value;
use crate::runtime_impl::value::{Record, Value};

/// bare "${id}" passes through the Γ_set record list as `Value::RecordList`
/// (dotted flat projection `${id.v}` aggregation has been removed; multi-value passing uses bare `${id}` only).
#[test]
fn test_resolve_with_value_bare_id_passes_through_record_list() {
    // Arrange: set id = record list (v=x, v=y) in Γ_set of scope
    let mut rec_x = Record::default();
    rec_x.fields.insert("v".to_string(), "x".to_string());
    let mut rec_y = Record::default();
    rec_y.fields.insert("v".to_string(), "y".to_string());
    let mut scope = empty_scope();
    // Place the record set producer on the env side (Dir). bare `${id}` is passed through as a RecordList via transparent get.
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "id",
        vec![rec_x.clone(), rec_y.clone()],
    );

    // Act
    let result = resolve_with_value("${id}", &scope);

    // Assert: Γ_set is passed through as RecordList
    assert_eq!(result, Value::RecordList(vec![rec_x, rec_y]));
}
