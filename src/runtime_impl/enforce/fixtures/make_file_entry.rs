use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Interval, Quant};

/// Helper that constructs a file `ExprEntry` for tests.
///
/// Passing `Some((0, Some(1)))` for `count` creates an optional entry equivalent (`Quant::Explicit { min: 0, max: Some(1) }`).
pub fn make_file_entry(
    file_pattern: ExprPattern,
    count: Option<(usize, Option<usize>)>,
) -> ExprEntry {
    let quant = match count {
        None => Quant::Default,
        Some((min, max)) => Quant::Explicit(Interval { min, max }),
    };
    ExprEntry {
        id: None,
        source_path: None,
        count: quant,
        matcher: ExprMatcher::File {
            pattern: file_pattern,
            subtree: ExprSubtree::Leaf,
        },
    }
}
