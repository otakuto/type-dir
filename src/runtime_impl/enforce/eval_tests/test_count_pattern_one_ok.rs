use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (a) Pattern count{1,2}: 1 match → within the interval, no error.
#[test]
fn test_count_pattern_one_ok() {
    // Arrange: 1 matching child
    let entry = count_file_regex(r"^a[0-9]+\.rs$", (1, Some(2)));
    let tree = tree_with(&["a1.rs"]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
