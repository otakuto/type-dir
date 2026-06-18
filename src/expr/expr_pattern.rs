use crate::yaml::RegexPattern;

/// Validated name pattern.
#[derive(Debug, Clone)]
pub enum ExprPattern {
    Exact(String),
    Regex(RegexPattern),
}

impl ExprPattern {
    /// Returns the structural count upper bound.
    ///
    /// `Exact` patterns match at most one node due to filesystem name uniqueness, so returns `Some(1)`.
    /// `Regex` patterns return `None` to indicate no upper bound.
    #[allow(dead_code)]
    pub fn structural_max(&self) -> Option<usize> {
        match self {
            ExprPattern::Exact(_) => Some(1),
            ExprPattern::Regex(_) => None,
        }
    }
}
