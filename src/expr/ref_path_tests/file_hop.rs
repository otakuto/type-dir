use crate::expr::{Hop, RefHead, parse_ref};

/// `${f.file.y}` — local reference with `Hop::File("y")`.
#[test]
fn file_hop() {
    // Arrange
    let key = "f.file.y";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(matches!(result.head, RefHead::Bare(ref h) if h == "f"));
    assert_eq!(result.hops, vec![Hop::File("y".to_string())]);
}
