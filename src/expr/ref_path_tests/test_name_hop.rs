use super::super::{Hop, RefHead, parse_ref};

/// `${f.name}` — `.name` is not a keyword; parses as `Hop::Field("name")` (legacy/invalid).
#[test]
fn name_as_legacy_field() {
    // Arrange
    let key = "f.name";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(matches!(result.head, RefHead::Bare(ref h) if h == "f"));
    assert_eq!(result.hops, vec![Hop::Field("name".to_string())]);
}
