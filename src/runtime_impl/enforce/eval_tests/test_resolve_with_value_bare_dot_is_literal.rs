use crate::runtime_impl::enforce::fixtures::empty_scope;
use crate::runtime_impl::enforce::with::resolve_with_value;
use crate::runtime_impl::value::Value;

/// A bare "foo.bar" is returned as Value::Scalar literal (not treated as an id.var reference).
#[test]
fn test_resolve_with_value_bare_dot_is_literal() {
    // Arrange
    let scope = empty_scope();

    // Act
    let result = resolve_with_value("foo.bar", &scope);

    // Assert
    assert_eq!(result, Value::Scalar("foo.bar".to_string()));
}
