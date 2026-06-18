use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprMatcher, Quant};
use crate::yaml::{RuleName, VarName};

/// Helper that builds a splice entry (`- use: rule.X`).
pub fn splice_entry(rule: &str, with_args: IndexMap<VarName, String>) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Use {
            rule: RuleName(rule.to_string()),
            with_args,
        },
    }
}
