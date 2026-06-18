use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Interval, Quant};
use crate::yaml::RegexPattern;

/// Helper that constructs a file entry with a count (Regex pattern).
pub fn count_file_regex(regex: &str, count: (usize, Option<usize>)) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Explicit(Interval {
            min: count.0,
            max: count.1,
        }),
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Regex(RegexPattern(regex.to_string())),
            subtree: ExprSubtree::Leaf,
        },
    }
}
