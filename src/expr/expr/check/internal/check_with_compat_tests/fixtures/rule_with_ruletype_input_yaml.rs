use crate::yaml::{RuleName, VarName, YamlEntry, YamlRule, YamlWithShape};
use indexmap::IndexMap;

/// Creates a rule with a `RuleType(R)` with declaration using `rule.<type_rule>` syntax.
///
/// `type_rule` must be a rule name (without the `rule.` prefix); the prefix is added here.
pub fn rule_with_ruletype_input_yaml(
    with_var: &str,
    type_rule: &str,
    entries: Vec<YamlEntry>,
) -> YamlRule {
    let qualified = format!("rule.{type_rule}");
    let value: serde_yaml::Value = serde_yaml::from_str(&qualified).expect("yaml parse failed");
    let mut with_params = IndexMap::new();
    with_params.insert(VarName(with_var.to_string()), YamlWithShape(value));
    YamlRule {
        rule: RuleName("".to_string()),
        with_params,
        note: None,
        body: entries,
    }
}
