use super::super::{Hop, RefHead, parse_ref};

/// `${fetch.dirs}` — bare fetch namespace (no tail hops).
#[test]
fn fetch_ns_bare() {
    // Arrange
    let key = "fetch.dirs";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::FetchNs { id, tail } => {
            assert_eq!(id, "dirs");
            assert!(tail.is_empty());
        }
        other => panic!("expected FetchNs, got {other:?}"),
    }
}

/// `${fetch.dirs.dir.node}` — fetch namespace with dir hop.
#[test]
fn fetch_ns_with_dir_hop() {
    // Arrange
    let key = "fetch.dirs.dir.node";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::FetchNs { id, tail } => {
            assert_eq!(id, "dirs");
            assert_eq!(tail, &vec![Hop::Dir("node".to_string())]);
        }
        other => panic!("expected FetchNs, got {other:?}"),
    }
}

/// `${fetch.dirs.regex.n}` — fetch namespace with regex hop.
#[test]
fn fetch_ns_with_regex_hop() {
    // Arrange
    let key = "fetch.dirs.regex.n";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::FetchNs { id, tail } => {
            assert_eq!(id, "dirs");
            assert_eq!(tail, &vec![Hop::Regex("n".to_string())]);
        }
        other => panic!("expected FetchNs, got {other:?}"),
    }
}

/// `${fetch.dirs.file.cfg}` — fetch namespace with file hop.
#[test]
fn fetch_ns_with_file_hop() {
    // Arrange
    let key = "fetch.dirs.file.cfg";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::FetchNs { id, tail } => {
            assert_eq!(id, "dirs");
            assert_eq!(tail, &vec![Hop::File("cfg".to_string())]);
        }
        other => panic!("expected FetchNs, got {other:?}"),
    }
}
