use crate::yaml::RuleName;
use serde::Deserialize;

use super::YamlRule;

/// Represents the entire configuration file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlConfig {
    pub version: u32,
    #[serde(default)]
    pub ignore: Vec<String>,
    /// The top-level rule definitions, each written as `- rule: <name>` (definition syntax).
    /// Invocations use `- use: rule.<name>` syntax; `rule:` is for definitions only.
    /// Name uniqueness is validated at compile time (`DuplicateRule`), not by the parser.
    #[serde(default)]
    pub rules: Vec<YamlRule>,
    /// Reference to the contents-only rule that serves as the start symbol (scalar).
    pub entry: RuleName,
}
