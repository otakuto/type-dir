use crate::runtime_impl::enforce::fixtures::{count_file_exact, run_entries, tree_with};

/// (b) Exact count{1,1}: present → no error.
#[test]
fn test_count_exact_present_ok() {
    // Arrange: file Exact "config.toml" with count{1,1}; child exists
    let entry = count_file_exact("config.toml", (1, Some(1)));
    let tree = tree_with(&["config.toml"]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
