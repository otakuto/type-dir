use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, Quant};
use crate::yaml::VarName;

/// Helper that constructs a `for` entry (Expr string variant).
pub fn make_for_entry_expr(var: &str, expr: &str, rules: Vec<ExprEntry>) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName(var.to_string()),
            source: ExprForSource::Expr(expr.to_string()),
            body: rules,
        },
    }
}
