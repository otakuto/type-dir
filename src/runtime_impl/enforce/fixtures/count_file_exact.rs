use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Interval, Quant};

/// Helper that constructs a file entry with a count (Exact pattern).
pub fn count_file_exact(name: &str, count: (usize, Option<usize>)) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Explicit(Interval {
            min: count.0,
            max: count.1,
        }),
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Exact(name.to_string()),
            subtree: ExprSubtree::Leaf,
        },
    }
}
