use crate::yaml::{RuleName, YamlEntry, YamlRule};
use indexmap::IndexMap;

pub fn make_rule(entries: Vec<YamlEntry>) -> YamlRule {
    YamlRule {
        rule: RuleName("".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: entries,
    }
}

#[allow(non_snake_case)]
pub fn RegexPatternFor(pattern: &str) -> crate::yaml::RegexPattern {
    crate::yaml::RegexPattern(pattern.to_string())
}
