use crate::runtime_impl::enforce::with::scope_to_scalar;
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::{Record, Value};

/// When scope contains a record value, `scope_to_scalar` expands both the main value (`var → r.whole()`)
/// and fields (`var.field → value`) (spec: bare `${var}` evaluates to the full match stored in fields["0"]).
#[test]
fn scope_to_scalar_expands_record_to_main_value_and_fields() {
    // Arrange: capture x=Record{fields{"0":"foo_mutation", "stem":"foo"}} (referenced via bare `${x}`)
    let mut scope = Scope::new();
    let mut rec = Record::default();
    rec.fields
        .insert("0".to_string(), "foo_mutation".to_string());
    rec.fields.insert("stem".to_string(), "foo".to_string());
    scope.bind_lex(NodeKind::Regex, "x", Value::Record(rec));

    // Act
    let result = scope_to_scalar(&scope);

    // Assert: since this is a bare capture, `x` expands to the match name and `x.stem` to the field value.
    assert_eq!(result.get("x"), Some(&"foo_mutation".to_string()));
    assert_eq!(result.get("x.stem"), Some(&"foo".to_string()));
}
