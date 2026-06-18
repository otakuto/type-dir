use super::super::{Hop, RefHead, parse_ref};

/// `${use.rec}` — bare use namespace (no tail hops).
#[test]
fn use_ns_bare() {
    // Arrange
    let key = "use.rec";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::UseNs { id, tail } => {
            assert_eq!(id, "rec");
            assert!(tail.is_empty());
        }
        other => panic!("expected UseNs, got {other:?}"),
    }
}

/// `${use.feature}` — bare use namespace with a different id.
#[test]
fn use_ns_bare_feature() {
    // Arrange
    let key = "use.feature";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::UseNs { id, tail } => {
            assert_eq!(id, "feature");
            assert!(tail.is_empty());
        }
        other => panic!("expected UseNs, got {other:?}"),
    }
}

/// `${use.rec.dir.node}` — use namespace with dir hop.
#[test]
fn use_ns_with_dir_hop() {
    // Arrange
    let key = "use.rec.dir.node";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::UseNs { id, tail } => {
            assert_eq!(id, "rec");
            assert_eq!(tail, &vec![Hop::Dir("node".to_string())]);
        }
        other => panic!("expected UseNs, got {other:?}"),
    }
}

/// `${use.rec.regex.stem}` — use namespace with regex hop.
#[test]
fn use_ns_with_regex_hop() {
    // Arrange
    let key = "use.rec.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::UseNs { id, tail } => {
            assert_eq!(id, "rec");
            assert_eq!(tail, &vec![Hop::Regex("stem".to_string())]);
        }
        other => panic!("expected UseNs, got {other:?}"),
    }
}
