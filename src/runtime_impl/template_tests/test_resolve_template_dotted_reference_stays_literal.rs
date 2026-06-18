use crate::runtime_impl::env::Scope;
use crate::runtime_impl::template::resolve_template;

/// A dotted reference `${id.var}` with no record binding in scope is kept as a literal in the pattern
/// (because set iteration is handled only by `for`).
#[test]
fn test_resolve_template_dotted_reference_stays_literal() {
    // Arrange
    let scope = Scope::new();

    // Act
    let result = resolve_template("${fe.component}.stories.tsx", &scope);

    // Assert — no record binding in scope, so the reference is kept as a literal.
    assert_eq!(result, "${fe.component}.stories.tsx");
}
