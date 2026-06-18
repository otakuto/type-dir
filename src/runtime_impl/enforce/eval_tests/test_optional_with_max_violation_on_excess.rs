use crate::error::LintError;
use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (g2) optional: true + max specified → exceeding max is an error.
#[test]
fn test_optional_with_max_violation_on_excess() {
    // Arrange: Quant::Explicit({ min: 0, max: Some(2) }), 3 matching children
    let entry = count_file_regex(r"^a[0-9]+\.rs$", (0, Some(2)));
    let tree = tree_with(&["a1.rs", "a2.rs", "a3.rs"]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert: CountViolation observed=3, max=Some(2)
    assert_eq!(errors.len(), 1, "expected 1 CountViolation: {:?}", errors);
    let LintError::CountViolation { observed, max, .. } = &errors[0] else {
        panic!("expected CountViolation: {:?}", errors[0]);
    };
    assert_eq!(*observed, 3);
    assert_eq!(*max, Some(2));
}
