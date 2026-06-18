use super::super::{Hop, RefHead, parse_ref};

/// `${group.g}` — bare group namespace (no tail hops).
#[test]
fn group_ns_bare() {
    // Arrange
    let key = "group.g";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::GroupNs { id, tail } => {
            assert_eq!(id, "g");
            assert!(tail.is_empty());
        }
        other => panic!("expected GroupNs, got {other:?}"),
    }
}

/// `${group.g.dir.node}` — group namespace with a dir hop.
#[test]
fn group_ns_with_dir_hop() {
    // Arrange
    let key = "group.g.dir.node";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::GroupNs { id, tail } => {
            assert_eq!(id, "g");
            assert_eq!(tail, &vec![Hop::Dir("node".to_string())]);
        }
        other => panic!("expected GroupNs, got {other:?}"),
    }
}
