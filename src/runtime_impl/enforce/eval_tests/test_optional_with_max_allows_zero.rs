use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (g) optional: true + max specified → effective {0, max} (0 matches is fine).
#[test]
fn test_optional_with_max_allows_zero() {
    // Arrange: file regex with Quant::Explicit({ min: 0, max: Some(2) }) (normalized result of optional:true + max:2)
    let entry = count_file_regex(r"^a[0-9]+\.rs$", (0, Some(2)));
    let tree = tree_with(&[]); // 0 matches

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert: min=0, so no errors
    assert!(
        errors.is_empty(),
        "optional+max with 0 matches should produce no errors: {:?}",
        errors
    );
}
