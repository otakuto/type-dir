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
