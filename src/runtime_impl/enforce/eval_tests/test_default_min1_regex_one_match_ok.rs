use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::fixtures::{run_entries, tree_with};
use crate::yaml::RegexPattern;

/// (e2) A `Quant::Default` regex entry with 1 or more matches → no error (satisfies {1,∞}).
#[test]
fn test_default_min1_regex_one_match_ok() {
    // Arrange: 1 matching child
    let entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Regex(RegexPattern(r"^a[0-9]+\.rs$".to_string())),
            subtree: ExprSubtree::Leaf,
        },
    };
    let tree = tree_with(&["a1.rs"]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
