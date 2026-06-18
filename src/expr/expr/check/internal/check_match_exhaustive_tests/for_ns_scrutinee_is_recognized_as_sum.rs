use super::super::check_match_exhaustive;
use super::fixtures::{config_with, for_ns_match, for_with_id_and_choice};

/// A match that scrutinizes a `for` variable iterating `${for.<id>}` produces no error when all
/// Sum constructors are covered.
///
/// Scenario:
///   - `for x in ${dirs} / id: classified / rules: [one_of id: kind :: [pair, single]]`
///   - `for r in ${for.classified} / rules: [match ${r} :: [pair, single]]`
///
/// The for-entry `classified` is the Sum; its constructors are lifted from the inner `one_of`'s
/// alternative ids [pair, single]. The match covers both arms → no error.
#[test]
fn for_ns_scrutinee_is_recognized_as_sum() {
    // Arrange
    let config = config_with(vec![
        for_with_id_and_choice("x", "dirs", "classified", "kind", &["pair", "single"]),
        for_ns_match("r", "classified", "r", &["pair", "single"]),
    ]);

    // Act
    let errors = check_match_exhaustive(&config);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
