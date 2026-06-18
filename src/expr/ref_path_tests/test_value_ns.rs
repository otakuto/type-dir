use super::super::{Hop, RefHead, parse_ref};

/// `${value.acc}` — bare `value.` namespace (no tail hops).
#[test]
fn value_ns_bare() {
    // Arrange
    let key = "value.acc";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ValueNs { var, tail } => {
            assert_eq!(var, "acc");
            assert!(tail.is_empty());
        }
        other => panic!("expected ValueNs, got {other:?}"),
    }
}

/// `${value.names}` — bare `value.` namespace used as a list (no tail hops).
#[test]
fn value_ns_list_bare() {
    // Arrange
    let key = "value.names";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ValueNs { var, tail } => {
            assert_eq!(var, "names");
            assert!(tail.is_empty());
        }
        other => panic!("expected ValueNs, got {other:?}"),
    }
}

/// `${value.acc.regex.x}` — `value.` namespace with a regex hop in the tail.
#[test]
fn value_ns_regex_hop() {
    // Arrange
    let key = "value.acc.regex.x";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::ValueNs { var, tail } => {
            assert_eq!(var, "acc");
            assert_eq!(tail, &vec![Hop::Regex("x".to_string())]);
        }
        other => panic!("expected ValueNs, got {other:?}"),
    }
}
