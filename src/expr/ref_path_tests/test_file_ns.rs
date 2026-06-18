use super::super::{Hop, RefHead, parse_ref};

/// `${file.cfg}` — bare file namespace (no tail hops).
#[test]
fn file_ns_bare() {
    // Arrange
    let key = "file.cfg";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::FileNs { id, tail } => {
            assert_eq!(id, "cfg");
            assert!(tail.is_empty());
        }
        other => panic!("expected FileNs, got {other:?}"),
    }
}

/// `${file.cfg.regex.stem}` — file namespace with a regex hop.
#[test]
fn file_ns_with_regex_hop() {
    // Arrange
    let key = "file.cfg.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::FileNs { id, tail } => {
            assert_eq!(id, "cfg");
            assert_eq!(tail, &vec![Hop::Regex("stem".to_string())]);
        }
        other => panic!("expected FileNs, got {other:?}"),
    }
}
