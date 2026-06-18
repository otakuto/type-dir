use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::runtime_impl::enforce::fixtures::{run_entries, tree_with};
use crate::yaml::RegexPattern;

/// (e) A `Quant::Default` (user-unspecified) regex entry with 0 matches → E019 from default min=1.
///
/// Default inversion: a regex entry with no count/min/max/optional specified in YAML has an
/// effective interval of {1,∞}; with 0 matching children, a CountViolation(E019) is emitted.
#[test]
fn test_default_min1_regex_zero_match_emits_count_violation() {
    // Arrange: file regex entry with count: Quant::Default (user-unspecified); 0 matching children
    let entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Regex(RegexPattern(r"^a[0-9]+\.rs$".to_string())),
            subtree: ExprSubtree::Leaf,
        },
    };
    let tree = tree_with(&[]);

    // Act
    let errors = run_entries(&[entry], &tree);

    // Assert: CountViolation(E019) from default min=1
    assert_eq!(errors.len(), 1, "expected 1 CountViolation: {:?}", errors);
    let LintError::CountViolation {
        observed, min, max, ..
    } = &errors[0]
    else {
        panic!("expected CountViolation: {:?}", errors[0]);
    };
    assert_eq!(*observed, 0);
    assert_eq!(*min, 1);
    assert_eq!(*max, None);
}
