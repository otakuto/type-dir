use crate::runtime_impl::enforce::fixtures::{choice_a_b, run_entry, tree_with};

/// choice {min:0, max:1}: only a exists → realized=1 within [0,1], no errors.
#[test]
fn test_choice_0_1_single_ok() {
    // Arrange
    let entry = choice_a_b(0, Some(1));
    let tree = tree_with(&["a"]);

    // Act
    let errors = run_entry(entry, &tree);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
