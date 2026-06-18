use crate::error::LintError;
use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (a) Pattern count{1,2}: 0 matches → E019 (observed=0 is below the lower bound).
#[test]
fn test_count_pattern_zero_violation() {
    // Arrange: file regex `^a[0-9]+\.rs$` with count{1,2}; 0 matching children
    let entry = count_file_regex(r"^a[0-9]+\.rs$", (1, Some(2)));
    let tree = tree_with(&[]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert
    assert_eq!(errors.len(), 1, "expected 1 CountViolation: {:?}", errors);
    let LintError::CountViolation {
        observed, min, max, ..
    } = &errors[0]
    else {
        panic!("expected CountViolation: {:?}", errors[0]);
    };
    assert_eq!(*observed, 0);
    assert_eq!(*min, 1);
    assert_eq!(*max, Some(2));
}
