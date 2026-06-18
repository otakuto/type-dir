use crate::yaml::RuleName;
use indexmap::IndexMap;

use super::common::{dir_id_entry, file_id_entry, for_items_entry, rule_with_ruletype_input};

/// Builds a two-rule config where:
/// - `producer` has one public id `it` (dir) with a child id `leaf` (file).
/// - `consumer` declares `items: producer` (RuleType) and iterates with `for x in ${choice.items}`.
pub fn build_rules_for_binding_dir_file_kind(
    body: Vec<crate::yaml::YamlEntry>,
) -> IndexMap<RuleName, crate::yaml::YamlRule> {
    // producer: dir: regex '^(?<stem>.+)$', id: it
    //   rules:
    //     - file: foo.txt, id: leaf
    let leaf_entry = file_id_entry("foo.txt", "leaf");
    let producer_entry = dir_id_entry(r"^(?<stem>.+)$", "it", vec![leaf_entry]);
    let producer_rule = crate::yaml::YamlRule {
        rule: RuleName("producer".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![producer_entry],
    };

    // consumer: with: items: producer, body: for x in ${choice.items} { body }
    let consumer_rule = rule_with_ruletype_input("items", "producer", vec![for_items_entry(body)]);

    let mut rules = IndexMap::new();
    rules.insert(RuleName("producer".to_string()), producer_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);
    rules
}
