use crate::runtime_impl::enforce::fixtures::{choice_a_b, run_entry, tree_with};

/// choice {min:0, max:0} (forbidden): no error when the target file is absent.
#[test]
fn test_choice_max_zero_empty_ok() {
    // Arrange
    let entry = choice_a_b(0, Some(0));
    let tree = tree_with(&[]);

    // Act
    let errors = run_entry(entry, &tree);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
