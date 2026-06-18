use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, Quant};

use super::make_file_entry::make_file_entry;

/// Helper that builds a Group entry for choice {min, max, of:[file a, file b]}.
pub fn choice_a_b(min: usize, max: Option<usize>) -> ExprEntry {
    let alt_a = make_file_entry(ExprPattern::Exact("a".to_string()), None);
    let alt_b = make_file_entry(ExprPattern::Exact("b".to_string()), None);
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min,
            max,
            body: vec![alt_a, alt_b],
        },
    }
}
