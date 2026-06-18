use crate::runtime_impl::env::{Scope, ScopeRef};
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::Record;

/// When Γ_lex has no matching id, the transparent `get` falls back to Γ_set and returns `ScopeRef::Set`.
#[test]
fn test_get_falls_back_to_sets_when_lex_absent() {
    // Arrange: register id "y" only on the env side (Dir).
    let mut scope = Scope::new();
    let rec = Record::default();
    scope.bind_env(NodeKind::Dir, "y", vec![rec.clone()]);

    // Act
    let result = scope.get("y");

    // Assert: the env side (Set) is returned.
    assert_eq!(result, Some(ScopeRef::Set(&[rec])));
}
