use super::super::{Hop, RefHead, parse_ref};

/// `${x.dir.a.file.b.regex.g}` — local reference with chained hops.
#[test]
fn chained_hops() {
    // Arrange
    let key = "x.dir.a.file.b.regex.g";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(matches!(result.head, RefHead::Bare(ref h) if h == "x"));
    assert_eq!(
        result.hops,
        vec![
            Hop::Dir("a".to_string()),
            Hop::File("b".to_string()),
            Hop::Regex("g".to_string()),
        ]
    );
}
