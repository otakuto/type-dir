use super::super::check_match_exhaustive;
use super::fixtures::{config_with, for_match, sum_group};
use crate::error::SemanticError;

/// A match missing an arm for one Sum tag produces E024 with that tag in `missing`.
#[test]
fn missing_arm_is_error() {
    // Arrange: items has alts {service, config}; the match only covers service.
    let config = config_with(vec![
        sum_group("items", &["service", "config"]),
        for_match("c", "items", "c", &["service"]),
    ]);

    // Act
    let errors = check_match_exhaustive(&config);

    // Assert
    assert_eq!(errors.len(), 1);
    let SemanticError::NonExhaustiveMatch {
        rule,
        scrutinee,
        missing,
        extra,
    } = &errors[0]
    else {
        panic!("unexpected: {:?}", errors[0]);
    };
    assert_eq!(rule, "root");
    assert_eq!(scrutinee, "c");
    assert_eq!(missing, &vec!["config".to_string()]);
    assert!(extra.is_empty());
}
