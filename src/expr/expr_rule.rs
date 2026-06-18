use indexmap::IndexMap;

use crate::expr::ExprEntry;
use crate::yaml::{VarName, WithShape};

/// Validated reusable rule definition. Treated as a macro for an entry sequence (content model).
#[derive(Debug, Clone)]
pub struct ExprRule {
    /// Parameter declarations (name → shape declaration). Carries scalar/record-set distinction,
    /// default value, and shape.
    pub with_params: IndexMap<VarName, WithShape>,
    /// Description of the rule (used in diagnostic output; `None` when not specified).
    pub note: Option<String>,
    pub rules: Vec<ExprEntry>,
}
