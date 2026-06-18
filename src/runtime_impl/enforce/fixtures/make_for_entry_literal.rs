use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, Quant};
use crate::yaml::VarName;

/// Helper that constructs a `for` entry (literal list variant).
pub fn make_for_entry_literal(var: &str, values: Vec<&str>, rules: Vec<ExprEntry>) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName(var.to_string()),
            source: ExprForSource::Literal(values.into_iter().map(|s| s.to_string()).collect()),
            body: rules,
        },
    }
}
