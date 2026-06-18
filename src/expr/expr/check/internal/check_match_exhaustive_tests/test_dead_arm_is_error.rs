use super::super::check_match_exhaustive;
use super::fixtures::{config_with, for_match, sum_group};
use crate::error::SemanticError;

/// A match with an arm whose id is not a Sum tag produces E024 with that id in `extra`.
#[test]
fn dead_arm_is_error() {
    // Arrange: items has alts {service, config}; the match adds a dead arm `ghost`.
    let config = config_with(vec![
        sum_group("items", &["service", "config"]),
        for_match("c", "items", "c", &["service", "config", "ghost"]),
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
    assert!(missing.is_empty());
    assert_eq!(extra, &vec!["ghost".to_string()]);
}
