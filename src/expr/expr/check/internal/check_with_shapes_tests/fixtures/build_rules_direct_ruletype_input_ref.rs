use crate::yaml::RuleName;
use indexmap::IndexMap;

use super::common::{dir_id_entry, file_id_entry, rule_with_ruletype_input};

/// Builds a two-rule config where:
/// - `producer` has one public id `it` (dir) with capture `stem` and child file id `leaf`.
/// - `consumer` declares `f: producer` (RuleType input) and uses the given body entries.
pub fn build_rules_direct_ruletype_input_ref(
    body: Vec<crate::yaml::YamlEntry>,
) -> IndexMap<RuleName, crate::yaml::YamlRule> {
    // producer: dir: regex '^(?<stem>.+)$', id: it
    //   rules:
    //     - file: foo.txt, id: leaf
    let leaf = file_id_entry("foo.txt", "leaf");
    let producer_entry = dir_id_entry(r"^(?<stem>.+)$", "it", vec![leaf]);
    let producer_rule = crate::yaml::YamlRule {
        rule: RuleName("producer".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![producer_entry],
    };
    // consumer: with: f: producer, body: <body>
    let consumer_rule = rule_with_ruletype_input("f", "producer", body);
    let mut rules = IndexMap::new();
    rules.insert(RuleName("producer".to_string()), producer_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);
    rules
}
