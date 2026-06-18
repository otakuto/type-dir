use super::super::{Hop, RefHead, parse_ref};

/// `${f.regex.stem}` — local reference with `Hop::Regex`.
#[test]
fn regex_hop() {
    // Arrange
    let key = "f.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(matches!(result.head, RefHead::Bare(ref h) if h == "f"));
    assert_eq!(result.hops, vec![Hop::Regex("stem".to_string())]);
}
