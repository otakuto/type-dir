use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::expr::ExprRule;
use crate::yaml::RuleName;

/// Validated configuration. Invalid combinations are excluded at the type level.
#[derive(Debug)]
pub struct ConfigExpr {
    /// Reference to the contents-only rule that serves as the start symbol (scalar).
    pub entry: RuleName,
    pub rules: IndexMap<RuleName, ExprRule>,
    pub ignore: Vec<String>,
}

/// Carries the set of errors found during configuration validation.
#[derive(Debug)]
pub struct ConfigErrors(pub Vec<SemanticError>);

impl std::fmt::Display for ConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} config errors found", self.0.len())
    }
}

impl std::error::Error for ConfigErrors {}
