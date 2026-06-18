use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::fixtures::{count_file_regex, run_entries, tree_with};

/// (c) A choice alternative with count{2,2}: realized with 2 matches (satisfies one_of).
#[test]
fn test_count_in_choice_alternative_realizes_with_two() {
    // Arrange: in one_of(choice[1,1]), alternative a has count{2,2}, alternative b has no count.
    // When a matches 2 times, alternative a is realized (Σ=1) and one_of is satisfied.
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
    let tree = tree_with(&["a1.rs", "a2.rs"]);

    // Act
    let errors = run_entries(&[group], &tree);

    // Assert: alternative a is realized with exactly 2 matches, satisfying one_of
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
