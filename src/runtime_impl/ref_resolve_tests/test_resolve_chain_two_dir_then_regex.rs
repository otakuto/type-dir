use std::sync::Arc;

use crate::expr::Hop;
use crate::runtime_impl::ref_resolve::{ChainValue, resolve_chain};
use crate::runtime_impl::value::Record;

/// A two-hop chain `[Dir("a"), Regex("g")]` descends into children and projects the named field.
#[test]
fn test_resolve_chain_two_dir_then_regex() {
    // Arrange
    let mut child_a = Record::default();
    child_a
        .fields
        .insert("0".to_string(), "child-a".to_string());
    child_a.fields.insert("g".to_string(), "v".to_string());

    let mut root = Record::default();
    root.fields.insert("0".to_string(), "root".to_string());
    root.children
        .insert("a".to_string(), vec![Arc::new(child_a)]);

    let hops = vec![Hop::Dir("a".to_string()), Hop::Regex("g".to_string())];

    // Act
    let result = resolve_chain(&root, &hops);

    // Assert
    assert_eq!(result, Some(ChainValue::Scalars(vec!["v".to_string()])));
}
