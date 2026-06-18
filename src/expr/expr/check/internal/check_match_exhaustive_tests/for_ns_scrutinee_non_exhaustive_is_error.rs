use super::super::check_match_exhaustive;
use super::fixtures::{config_with, for_ns_match, for_with_id_and_choice};
use crate::error::SemanticError;

/// A match that scrutinizes a `for` variable iterating `${for.<id>}` but is missing an arm
/// produces E024 (NonExhaustiveMatch).
///
/// Scenario:
///   - `for x in ${dirs} / id: classified / rules: [one_of id: kind :: [pair, single]]`
///   - `for r in ${for.classified} / rules: [match ${r} :: [pair]]`   ← `single` is missing
#[test]
fn for_ns_scrutinee_non_exhaustive_is_error() {
    // Arrange
    let config = config_with(vec![
        for_with_id_and_choice("x", "dirs", "classified", "kind", &["pair", "single"]),
        for_ns_match("r", "classified", "r", &["pair"]),
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
    assert_eq!(scrutinee, "r");
    assert_eq!(missing, &vec!["single".to_string()]);
    assert!(extra.is_empty());
}
