use super::super::{Hop, RefHead, parse_ref};

/// `${rule.R.dir.x}` — `rule.` namespace: rule_id=R, tail=[Dir("x")].
#[test]
fn rule_ns_with_dir_hop() {
    // Arrange
    let key = "rule.R.dir.x";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::RuleNs { rule_id, tail } => {
            assert_eq!(rule_id, "R");
            assert_eq!(tail, &vec![Hop::Dir("x".to_string())]);
        }
        other => panic!("expected RuleNs, got {other:?}"),
    }
}

/// `${rule.feature.dir.feature_name}` — real-world splice reference.
#[test]
fn rule_ns_feature_dir_feature_name() {
    // Arrange
    let key = "rule.feature.dir.feature_name";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::RuleNs { rule_id, tail } => {
            assert_eq!(rule_id, "feature");
            assert_eq!(tail, &vec![Hop::Dir("feature_name".to_string())]);
        }
        other => panic!("expected RuleNs, got {other:?}"),
    }
}

/// `${rule.R}` — bare rule namespace (no tail hops).
#[test]
fn rule_ns_bare() {
    // Arrange
    let key = "rule.R";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::RuleNs { rule_id, tail } => {
            assert_eq!(rule_id, "R");
            assert!(tail.is_empty());
        }
        other => panic!("expected RuleNs, got {other:?}"),
    }
}

/// `${rule.R.regex.stem}` — rule namespace with regex hop.
#[test]
fn rule_ns_with_regex_hop() {
    // Arrange
    let key = "rule.R.regex.stem";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::RuleNs { rule_id, tail } => {
            assert_eq!(rule_id, "R");
            assert_eq!(tail, &vec![Hop::Regex("stem".to_string())]);
        }
        other => panic!("expected RuleNs, got {other:?}"),
    }
}
