use super::super::{Hop, RefHead, parse_ref};

/// `${with.feature_dir.regex.stem}` — `with.` namespace: param=feature_dir, tail=[Regex("stem")].
#[test]
fn with_ns_regex_stem() {
    // Arrange
    let key = "with.feature_dir.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::WithNs { param, tail } => {
            assert_eq!(param, "feature_dir");
            assert_eq!(tail, &vec![Hop::Regex("stem".to_string())]);
        }
        other => panic!("expected WithNs, got {other:?}"),
    }
}

/// `${with.feature_dir.file.project}` — `with.` namespace with file hop.
#[test]
fn with_ns_file_hop() {
    // Arrange
    let key = "with.feature_dir.file.project";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::WithNs { param, tail } => {
            assert_eq!(param, "feature_dir");
            assert_eq!(tail, &vec![Hop::File("project".to_string())]);
        }
        other => panic!("expected WithNs, got {other:?}"),
    }
}

/// `${with.features}` — bare `with.` namespace (no tail hops).
#[test]
fn with_ns_bare() {
    // Arrange
    let key = "with.features";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::WithNs { param, tail } => {
            assert_eq!(param, "features");
            assert!(tail.is_empty());
        }
        other => panic!("expected WithNs, got {other:?}"),
    }
}

/// `${with.feature_dir.dir.feature_dir}` — `with.` namespace with dir hop.
#[test]
fn with_ns_dir_hop() {
    // Arrange
    let key = "with.feature_dir.dir.feature_dir";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::WithNs { param, tail } => {
            assert_eq!(param, "feature_dir");
            assert_eq!(tail, &vec![Hop::Dir("feature_dir".to_string())]);
        }
        other => panic!("expected WithNs, got {other:?}"),
    }
}
