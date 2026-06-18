use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::template::resolve_template;
use crate::runtime_impl::value::{Record, Value};

/// A bare `${x}` evaluates to the full match stored in `fields["0"]` (spec 2).
#[test]
fn test_resolve_template_record_main_value_is_whole() {
    // Arrange: x=Record{fields{"0":"foo_mutation", "stem":"foo"}}
    let mut scope = Scope::new();
    let mut rec = Record::default();
    rec.fields
        .insert("0".to_string(), "foo_mutation".to_string());
    rec.fields.insert("stem".to_string(), "foo".to_string());
    scope.bind_lex(NodeKind::Value, "x", Value::Record(rec));

    // Act: bare `${x}` expands to the full match (fields["0"]), not to other fields.
    let result = resolve_template("${x}_handler.rs", &scope);

    // Assert
    assert_eq!(result, "foo_mutation_handler.rs".to_string());
}
