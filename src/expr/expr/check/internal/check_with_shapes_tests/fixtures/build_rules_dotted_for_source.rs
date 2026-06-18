use crate::yaml::RuleName;
use indexmap::IndexMap;

use super::common::{dir_id_entry, for_expr_entry, rule_with_ruletype_input};

/// Builds a two-rule config where:
/// - `producer` has public id `it` (dir) with child dir id `sub` (regex `^(?<name>.+)$`).
/// - `consumer` declares `f: producer` (RuleType) and uses
///   `for x in ${f.dir.sub} { <for_body> }`.
pub fn build_rules_dotted_for_source(
    for_body: Vec<crate::yaml::YamlEntry>,
) -> IndexMap<RuleName, crate::yaml::YamlRule> {
    // sub: dir: regex '^(?<name>.+)$', id: sub (no children)
    let sub_entry = dir_id_entry(r"^(?<name>.+)$", "sub", vec![]);
    // it: dir: regex '^(?<stem>.+)$', id: it, rules: [sub_entry]
    let it_entry = dir_id_entry(r"^(?<stem>.+)$", "it", vec![sub_entry]);
    let producer_rule = crate::yaml::YamlRule {
        rule: RuleName("producer".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![it_entry],
    };

    // consumer: with: f: producer
    //   rules:
    //     - for: x
    //       in: ${f.dir.sub}
    //       rules: <for_body>
    let for_entry = for_expr_entry("x", "${f.dir.sub}", for_body);
    let consumer_rule = rule_with_ruletype_input("f", "producer", vec![for_entry]);

    let mut rules = IndexMap::new();
    rules.insert(RuleName("producer".to_string()), producer_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);
    rules
}
