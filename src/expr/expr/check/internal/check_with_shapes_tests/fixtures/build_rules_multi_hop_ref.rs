use crate::yaml::RuleName;
use indexmap::IndexMap;

use super::common::{dir_id_entry, rule_with_ruletype_input};

/// Builds a two-rule config where:
/// - `producer` has public id `it` (dir) with child dir id `a` (regex `^(?<g>.+)$`).
/// - `consumer` declares `f: producer` (RuleType) and uses the given body entries.
pub fn build_rules_multi_hop_ref(
    body: Vec<crate::yaml::YamlEntry>,
) -> IndexMap<RuleName, crate::yaml::YamlRule> {
    // a: dir: regex '^(?<g>.+)$', id: a (no children)
    let a_entry = dir_id_entry(r"^(?<g>.+)$", "a", vec![]);
    // it: dir: regex '^[a-z]+$', id: it, rules: [a_entry]
    let it_entry = dir_id_entry(r"^[a-z]+$", "it", vec![a_entry]);
    let producer_rule = crate::yaml::YamlRule {
        rule: RuleName("producer".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![it_entry],
    };

    // consumer: with: f: producer, body: <body>
    let consumer_rule = rule_with_ruletype_input("f", "producer", body);

    let mut rules = IndexMap::new();
    rules.insert(RuleName("producer".to_string()), producer_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);
    rules
}
