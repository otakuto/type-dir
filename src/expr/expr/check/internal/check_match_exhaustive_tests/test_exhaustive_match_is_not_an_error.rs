use super::super::check_match_exhaustive;
use super::fixtures::{config_with, for_match, sum_group};

/// A match that covers exactly the Sum's alternative ids produces no error.
#[test]
fn exhaustive_match_is_not_an_error() {
    // Arrange: items has alts {service, config}; the match covers both.
    let config = config_with(vec![
        sum_group("items", &["service", "config"]),
        for_match("c", "items", "c", &["service", "config"]),
    ]);

    // Act
    let errors = check_match_exhaustive(&config);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
