use crate::error::LintError;
use crate::expr::{
    ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprSubtree, Interval, Quant,
};
use crate::runtime_impl::enforce::fixtures::{run_entries, tree_with};
use crate::yaml::{RegexPattern, VarName};

/// (d) The count of entries inside a `for` is evaluated independently per binding.
#[test]
fn test_count_in_for_is_independent_per_binding() {
    // Arrange: for x in ["a", "b"] { file '${x}.rs' count{2,2} }.
    // Each binding would require 2 copies of `a.rs` / `b.rs`, but an Exact name can only appear once,
    // so count{2,2} would be impossible. Here we use Regex to require 2 per binding, placing only the a-side 2.
    // x=a: 2 .rs files starting with "a" (satisfied); x=b: 0 .rs files starting with "b" (violation).
    let inner = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Explicit(Interval::exactly(2)),
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Regex(RegexPattern(r"^${value.x}[0-9]+\.rs$".to_string())),
            subtree: ExprSubtree::Leaf,
        },
    };
    let for_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("x".to_string()),
            source: ExprForSource::Literal(vec!["a".to_string(), "b".to_string()]),
            body: vec![inner],
        },
    };
    // a1.rs / a2.rs match the x=a binding (satisfied with 2). The b-side has 0 matches (violation).
    let tree = tree_with(&["a1.rs", "a2.rs"]);

    // Act
    let errors = run_entries(&[for_entry], &tree);

    // Assert: x=b binding's count{2,2} is violated with observed=0 (x=a is satisfied so no a-side violation)
    let count_violations: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, LintError::CountViolation { .. }))
        .collect();
    assert_eq!(
        count_violations.len(),
        1,
        "only x=b binding should have count violation: {:?}",
        errors
    );
    let LintError::CountViolation { observed, .. } = count_violations[0] else {
        unreachable!()
    };
    assert_eq!(*observed, 0, "x=b binding observed count is 0");
}
