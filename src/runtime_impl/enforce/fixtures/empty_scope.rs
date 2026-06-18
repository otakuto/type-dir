use crate::runtime_impl::env::Scope;

/// Helper that returns an empty scope.
pub fn empty_scope() -> Scope {
    Scope::new()
}
