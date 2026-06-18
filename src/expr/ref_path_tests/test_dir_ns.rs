use super::super::{Hop, RefHead, parse_ref};

/// `${dir.x}` — bare dir namespace (no tail hops).
#[test]
fn dir_ns_bare() {
    // Arrange
    let key = "dir.x";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert!(tail.is_empty());
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}

/// `${dir.x.file.y}` — dir namespace with a nested file hop (path reference).
#[test]
fn dir_ns_with_file_hop() {
    // Arrange
    let key = "dir.x.file.y";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert_eq!(tail, &vec![Hop::File("y".to_string())]);
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}

/// `${dir.x.regex.stem}` — dir namespace with a regex hop.
#[test]
fn dir_ns_with_regex_hop() {
    // Arrange
    let key = "dir.x.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert_eq!(tail, &vec![Hop::Regex("stem".to_string())]);
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}
