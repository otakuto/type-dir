use crate::error::LintError;
use crate::runtime_impl::enforce::fixtures::{count_file_exact, run_entries, tree_with};

/// (b) Exact count{1,1}: absent → unified into CountViolation(E019), not MissingRequired(E002).
#[test]
fn test_count_exact_absent_emits_count_violation_not_required() {
    // Arrange: file Exact "config.toml" with count{1,1}; no children
    let entry = count_file_exact("config.toml", (1, Some(1)));
    let tree = tree_with(&[]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert: report only 1 E019; no E002 (MissingRequired)
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    assert!(
        matches!(&errors[0], LintError::CountViolation { .. }),
        "expected CountViolation (not MissingRequired): {:?}",
        errors[0]
    );
    assert!(
        !errors
            .iter()
            .any(|e| matches!(e, LintError::MissingRequired { .. })),
        "must not double-report with MissingRequired: {:?}",
        errors
    );
}
