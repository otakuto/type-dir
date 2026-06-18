use crate::error::LintError;
use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (f) Only max specified (no optional/min) → effective {1, max} (default min=1 is applied).
#[test]
fn test_max_only_implies_min1() {
    // Arrange: file regex with Quant::Explicit({ min: 1, max: Some(2) }) (representing max-only intent in the final ExprEntry)
    // At compile time, max-only → normalized to min=1, so effective {1,2}.
    let entry = count_file_regex(r"^a[0-9]+\.rs$", (1, Some(2)));
    let tree = tree_with(&[]); // 0 matches → violates min=1

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert: CountViolation observed=0, min=1, max=Some(2)
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
