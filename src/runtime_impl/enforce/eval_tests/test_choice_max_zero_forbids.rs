use crate::error::LintError;
use crate::runtime_impl::enforce::fixtures::{choice_a_b, run_entry, tree_with};

/// choice {min:0, max:0} (forbidden): when the target file exists, realized=1 exceeds max=0, E013.
#[test]
fn test_choice_max_zero_forbids() {
    // Arrange
    let entry = choice_a_b(0, Some(0));
    let tree = tree_with(&["a"]);

    // Act
    let errors = run_entry(entry, &tree);

    // Assert
    assert_eq!(
        errors.len(),
        1,
        "expected 1 forbidden violation: {:?}",
        errors
    );
    let LintError::CardinalityViolation { realized, max, .. } = &errors[0] else {
        panic!("expected CardinalityViolation: {:?}", errors[0]);
    };
    assert_eq!(*realized, 1);
    assert_eq!(*max, Some(0));
}
