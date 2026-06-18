use super::super::{Hop, RefHead, parse_ref};

/// `${choice.c}` — bare choice namespace (no tail hops).
#[test]
fn choice_ns_bare() {
    // Arrange
    let key = "choice.c";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ChoiceNs { id, tail } => {
            assert_eq!(id, "c");
            assert!(tail.is_empty());
        }
        other => panic!("expected ChoiceNs, got {other:?}"),
    }
}

/// `${choice.c.file.y}` — choice namespace with a file hop.
#[test]
fn choice_ns_with_file_hop() {
    // Arrange
    let key = "choice.c.file.y";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ChoiceNs { id, tail } => {
            assert_eq!(id, "c");
            assert_eq!(tail, &vec![Hop::File("y".to_string())]);
        }
        other => panic!("expected ChoiceNs, got {other:?}"),
    }
}
