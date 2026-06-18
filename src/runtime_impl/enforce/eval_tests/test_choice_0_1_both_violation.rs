use crate::error::LintError;
use crate::runtime_impl::enforce::fixtures::{choice_a_b, run_entry, tree_with};

/// choice {min:0, max:1}: both a and b exist → realized=2 exceeds max=1, E013.
#[test]
fn test_choice_0_1_both_violation() {
    // Arrange
    let entry = choice_a_b(0, Some(1));
    let tree = tree_with(&["a", "b"]);

    // Act
    let errors = run_entry(entry, &tree);

    // Assert
    assert_eq!(
        errors.len(),
        1,
        "expected 1 CardinalityViolation: {:?}",
        errors
    );
    let LintError::CardinalityViolation {
        realized, min, max, ..
    } = &errors[0]
    else {
        panic!("expected CardinalityViolation: {:?}", errors[0]);
    };
    assert_eq!(*realized, 2);
    assert_eq!(*min, 0);
    assert_eq!(*max, Some(1));
}
