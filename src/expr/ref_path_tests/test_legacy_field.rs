use super::super::{Hop, RefHead, parse_ref};

/// `${f.stem}` — bare local with unqualified field produces `RefHead::Bare` + `Hop::Field`.
#[test]
fn legacy_field() {
    // Arrange
    let key = "f.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(matches!(result.head, RefHead::Bare(ref h) if h == "f"));
    assert_eq!(result.hops, vec![Hop::Field("stem".to_string())]);
}
