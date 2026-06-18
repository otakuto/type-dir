use crate::expr::{Hop, RefHead, parse_ref};

/// `${f.dir.x}` — local reference with `Hop::Dir("x")`.
#[test]
fn dir_hop() {
    // Arrange
    let key = "f.dir.x";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(matches!(result.head, RefHead::Bare(ref h) if h == "f"));
    assert_eq!(result.hops, vec![Hop::Dir("x".to_string())]);
}
