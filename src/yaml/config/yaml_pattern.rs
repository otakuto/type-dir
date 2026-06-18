#[cfg(test)]
#[path = "yaml_pattern_tests/tests.rs"]
mod tests;

use crate::yaml::RegexPattern;
use serde::Deserialize;

/// Represents a name match for dir/file.
/// A string is interpreted as a template (exact match); a map is interpreted as `PatternSpec`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum YamlPattern {
    Exact(String),
    Spec(PatternSpec),
}

impl YamlPattern {
    /// Returns whether this is the `Exact` variant.
    ///
    /// Consolidates the duplicated `matches!` checks in `compile.rs` and `check_entry_combination.rs`.
    pub fn is_exact(&self) -> bool {
        matches!(self, YamlPattern::Exact(_))
    }
}

/// Detailed pattern specification. `regex` is required.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternSpec {
    /// Regex pattern.
    #[serde(default)]
    pub regex: Option<RegexPattern>,
}
