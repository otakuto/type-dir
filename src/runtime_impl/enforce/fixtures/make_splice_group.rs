use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, Quant};
use crate::yaml::RuleName;

/// Helper that constructs a Group entry with Splice alternatives.
pub fn make_splice_group(min: usize, max: Option<usize>, rule_names: &[&str]) -> ExprEntry {
    let alternatives = rule_names
        .iter()
        .map(|name| ExprEntry {
            id: None,
            source_path: None,
            count: Quant::Default,
            matcher: ExprMatcher::Use {
                rule: RuleName(name.to_string()),
                with_args: IndexMap::new(),
            },
        })
        .collect();
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Choice {
            min,
            max,
            body: alternatives,
        },
    }
}
