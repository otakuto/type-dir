use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprSubtree, Interval, Quant};

/// Helper that constructs a dir `ExprEntry` for tests.
///
/// Passing `Some((0, Some(1)))` for `count` creates an optional entry equivalent (`Quant::Explicit { min: 0, max: Some(1) }`).
pub fn make_dir_entry(
    dir_pattern: ExprPattern,
    count: Option<(usize, Option<usize>)>,
    subtree: ExprSubtree,
) -> ExprEntry {
    let quant = match count {
        None => Quant::Default,
        Some((min, max)) => Quant::Explicit(Interval { min, max }),
    };
    ExprEntry {
        id: None,
        source_path: None,
        count: quant,
        matcher: ExprMatcher::Dir {
            pattern: dir_pattern,
            subtree,
        },
    }
}
