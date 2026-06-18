use crate::runtime_impl::env::{Scope, ScopeRef};
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::{Record, Value};

/// When the same id exists in both lex and env within the same frame, the transparent `get` gives
/// priority to lex.
///
/// This resolution order matches the behavior of `resolve_bare_key` (expand.rs): the for source
/// `${x}` first looks up the lexical binding x, and the id producer set x is shadowed. lex and env
/// are stored in separate maps keyed by `(kind, id)`, but `get` searches lex first using id alone
/// (kind-undetermined), which preserves this priority.
#[test]
fn test_lex_takes_priority_over_sets_on_same_key() {
    // Arrange: register id "x" as a scalar on the lex side (Regex) and as a record set on the env side (Dir).
    let mut scope = Scope::new();
    scope.bind_lex(NodeKind::Regex, "x", Value::Scalar("lex_value".to_string()));
    let rec = Record::default();
    scope.bind_env(NodeKind::Dir, "x", vec![rec]);

    // Act
    let result = scope.get("x");

    // Assert: the lex side (Scalar) is returned; the env side is shadowed.
    assert_eq!(
        result,
        Some(ScopeRef::Lex(&Value::Scalar("lex_value".to_string())))
    );
}
