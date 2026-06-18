use std::sync::Arc;

use crate::expr::Hop;
use crate::runtime_impl::ref_resolve::{ChainValue, resolve_chain};
use crate::runtime_impl::value::Record;

/// A two-hop chain `[Dir("a"), Dir("b")]` descends two levels and returns a `Records` set.
#[test]
fn test_resolve_chain_dir_then_dir() {
    // Arrange
    let grandchild = Record::default();
    let mut child_a = Record::default();
    child_a
        .children
        .insert("b".to_string(), vec![Arc::new(grandchild.clone())]);

    let mut root = Record::default();
    root.children
        .insert("a".to_string(), vec![Arc::new(child_a)]);

    let hops = vec![Hop::Dir("a".to_string()), Hop::Dir("b".to_string())];

    // Act
    let result = resolve_chain(&root, &hops);

    // Assert
    assert_eq!(result, Some(ChainValue::Records(vec![grandchild])));
}
