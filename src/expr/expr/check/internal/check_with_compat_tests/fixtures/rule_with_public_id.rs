use crate::yaml::YamlRule;

use super::common::{empty_rule, id_file_entry};

/// Creates a rule that has one public id (for use as a RuleType target).
pub fn rule_with_public_id(id: &str, regex: &str) -> YamlRule {
    let entry = id_file_entry(id, regex);
    empty_rule(vec![entry])
}
