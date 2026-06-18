use crate::yaml::{RuleName, YamlRule};
use indexmap::IndexMap;

use super::common::{any_of_sum_entry, dir_alt_entry, file_alt_entry, for_expr_entry, match_entry};

/// Builds a single `root` rule whose body contains:
///   - `any_of: id: items, of: [- id: service / dir: regex(svc), - id: config / file: regex(cfg)]`
///   - `for c in ${choice.items} / rules: [match: ${c} / rules: <arms>]`
pub fn build_rules_match_arm_sum_narrowing(
    arms: Vec<crate::yaml::YamlEntry>,
) -> IndexMap<RuleName, YamlRule> {
    // Sum group: any_of { id: items, of: [service alt, config alt] }
    let service_alt = dir_alt_entry("service", r"^(?<svc>.+)-service$");
    let config_alt = file_alt_entry("config", r"^(?<cfg>.+)\.conf$");
    let sum_entry = any_of_sum_entry("items", vec![service_alt, config_alt]);

    // for c in ${choice.items} { match: ${c} { arms... } }
    let match_e = match_entry("c", arms);
    let for_e = for_expr_entry("c", "${choice.items}", vec![match_e]);

    let root_rule = YamlRule {
        rule: RuleName("root".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![sum_entry, for_e],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("root".to_string()), root_rule);
    rules
}
