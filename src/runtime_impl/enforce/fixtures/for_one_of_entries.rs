use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, ExprPattern, Quant};
use crate::yaml::VarName;

use super::make_file_entry::make_file_entry;

/// Builds an entry list with `one_of: ['${value.x}_1.txt', '${value.x}_2.txt']` inside
/// `for {id: x, value: ["a","b"]}` rules. The one_of (min=1, max=1) is expanded per for binding.
pub fn for_one_of_entries() -> Vec<ExprEntry> {
    let alt1 = make_file_entry(ExprPattern::Exact("${value.x}_1.txt".to_string()), None);
    let alt2 = make_file_entry(ExprPattern::Exact("${value.x}_2.txt".to_string()), None);
    let group_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min: 1,
            max: Some(1),
            body: vec![alt1, alt2],
        },
    };
    let for_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("x".to_string()),
            source: ExprForSource::Literal(vec!["a".to_string(), "b".to_string()]),
            body: vec![group_entry],
        },
    };
    vec![for_entry]
}
