use std::sync::Arc;

use crate::expr::Hop;
use crate::runtime_impl::ref_resolve::{ChainValue, resolve_chain};
use crate::runtime_impl::value::Record;

/// `[Dir("a"), Dir("b")]` flat-maps over multiple `a` children, each having their own `b` children.
#[test]
fn test_resolve_chain_flatten_over_set() {
    // Arrange
    let b1 = Record::default();
    let b2 = Record::default();

    let mut a1 = Record::default();
    a1.children
        .insert("b".to_string(), vec![Arc::new(b1.clone())]);

    let mut a2 = Record::default();
    a2.children
        .insert("b".to_string(), vec![Arc::new(b2.clone())]);

    let mut root = Record::default();
    root.children
        .insert("a".to_string(), vec![Arc::new(a1), Arc::new(a2)]);

    let hops = vec![Hop::Dir("a".to_string()), Hop::Dir("b".to_string())];

    // Act
    let result = resolve_chain(&root, &hops);

    // Assert
    assert_eq!(result, Some(ChainValue::Records(vec![b1, b2])));
}
