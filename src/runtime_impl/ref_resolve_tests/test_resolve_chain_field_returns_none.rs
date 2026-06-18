use crate::expr::Hop;
use crate::runtime_impl::ref_resolve::resolve_chain;
use crate::runtime_impl::value::Record;

/// A chain containing `Hop::Field` anywhere returns `None` (legacy hops are not composable).
#[test]
fn test_resolve_chain_field_returns_none() {
    // Arrange
    let rec = Record::default();
    let hops = vec![Hop::Field("x".to_string())];

    // Act
    let result = resolve_chain(&rec, &hops);

    // Assert
    assert_eq!(result, None);
}
