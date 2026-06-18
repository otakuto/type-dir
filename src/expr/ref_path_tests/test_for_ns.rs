use super::super::{Hop, RefHead, parse_ref};

/// `${for.loop1}` — bare for namespace (no tail hops).
#[test]
fn for_ns_bare() {
    // Arrange
    let key = "for.loop1";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ForNs { id, tail } => {
            assert_eq!(id, "loop1");
            assert!(tail.is_empty());
        }
        other => panic!("expected ForNs, got {other:?}"),
    }
}

/// `${for.loop1.dir.node}` — for namespace with dir hop.
#[test]
fn for_ns_with_dir_hop() {
    // Arrange
    let key = "for.loop1.dir.node";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ForNs { id, tail } => {
            assert_eq!(id, "loop1");
            assert_eq!(tail, &vec![Hop::Dir("node".to_string())]);
        }
        other => panic!("expected ForNs, got {other:?}"),
    }
}

/// `${for.loop1.regex.stem}` — for namespace with regex hop.
#[test]
fn for_ns_with_regex_hop() {
    // Arrange
    let key = "for.loop1.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ForNs { id, tail } => {
            assert_eq!(id, "loop1");
            assert_eq!(tail, &vec![Hop::Regex("stem".to_string())]);
        }
        other => panic!("expected ForNs, got {other:?}"),
    }
}

/// `${for.loop1.file.cfg}` — for namespace with file hop.
#[test]
fn for_ns_with_file_hop() {
    // Arrange
    let key = "for.loop1.file.cfg";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ForNs { id, tail } => {
            assert_eq!(id, "loop1");
            assert_eq!(tail, &vec![Hop::File("cfg".to_string())]);
        }
        other => panic!("expected ForNs, got {other:?}"),
    }
}
