use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::template::resolve_template;
use crate::runtime_impl::value::Value;

/// A scalar-only template returns a single resolved string.
#[test]
fn test_resolve_template_scalar_only() {
    // Arrange
    let mut scope = Scope::new();
    scope.bind_lex(
        NodeKind::Regex,
        "layer",
        Value::Scalar("usecase".to_string()),
    );

    // Act
    let result = resolve_template("myapp-${layer}-foo", &scope);

    // Assert
    assert_eq!(result, "myapp-usecase-foo");
}
