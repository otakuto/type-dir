use super::super::check_match_exhaustive;
use super::fixtures::{config_with, for_match};
use crate::error::SemanticError;

/// A match whose scrutinee iterates a source that is not an id-bearing Group produces E025.
#[test]
fn scrutinee_not_a_sum_is_error() {
    // Arrange: no id-bearing group named `items` exists, so `c` is not a Sum.
    let config = config_with(vec![for_match("c", "items", "c", &["service", "config"])]);

    // Act
    let errors = check_match_exhaustive(&config);

    // Assert
    assert_eq!(errors.len(), 1);
    let SemanticError::MatchOnNonSum { rule, scrutinee } = &errors[0] else {
        panic!("unexpected: {:?}", errors[0]);
    };
    assert_eq!(rule, "root");
    assert_eq!(scrutinee, "c");
}
