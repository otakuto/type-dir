use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::template::resolve_template;
use crate::runtime_impl::value::{Record, Value};

/// Expands a field of a variable bound as a record value in scope via `${x.regex.name}`.
#[test]
fn test_resolve_template_record_field_reference() {
    // Arrange
    let mut scope = Scope::new();
    let mut rec = Record::default();
    rec.fields.insert("name".to_string(), "foo".to_string());
    // Place the record binding on the lex side (Value). The bare `${x.regex.name}` resolves via transparent get.
    scope.bind_lex(NodeKind::Value, "x", Value::Record(rec));

    // Act
    let result = resolve_template("${x.regex.name}_handler", &scope);

    // Assert
    assert_eq!(result, "foo_handler".to_string());
}
