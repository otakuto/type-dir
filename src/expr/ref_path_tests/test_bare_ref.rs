use super::super::{Ref, RefHead, parse_ref};

/// `${f}` — no hops: bare local reference.
#[test]
fn bare_ref() {
    // Arrange
    let key = "f";

    // Act
    let result = parse_ref(key);

    // Assert
    assert_eq!(
        result,
        Ref {
            head: RefHead::Bare("f".to_string()),
            hops: vec![]
        }
    );
}
