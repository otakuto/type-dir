use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (c) A choice alternative with count{2,2}: not realized with 1 match (below lower bound, one_of not satisfied).
#[test]
fn test_count_in_choice_alternative_not_realized_with_one() {
    // Arrange: alternative a has count{2,2} but only 1 match.
    // a is not realized, b is absent, so Σ=0 < min=1 → CardinalityViolation.
    let alt_a = count_file_regex(r"^a[0-9]+\.rs$", (2, Some(2)));
    let alt_b = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact("b.rs".to_string()),
            subtree: ExprSubtree::Leaf,
        },
    };
    let group = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt_a, alt_b],
        },
    };
    let tree = tree_with(&["a1.rs"]);

    // Act
    let errors = run_entries(&[group], &tree);

    // Assert: alternative a is not realized (c_a=1 ∉ [2,2]), Σ=0; one_of not satisfied
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, LintError::CardinalityViolation { realized: 0, .. })),
        "expected CardinalityViolation with realized=0: {:?}",
        errors
    );
}
