use indexmap::IndexMap;

use crate::expr::{ExprPattern, ExprRule};
use crate::yaml::RuleName;

use super::make_file_entry::make_file_entry;

/// Helper that constructs minimal rule definitions for resource_dir and resource_group_dir.
pub fn content_choice_rules() -> IndexMap<RuleName, ExprRule> {
    // resource_dir: file "res.toml" is required
    let resource_dir_rule = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_file_entry(
            ExprPattern::Exact("res.toml".to_string()),
            None,
        )],
    };
    // resource_group_dir: file "group.toml" is required
    let resource_group_dir_rule = ExprRule {
        with_params: IndexMap::new(),
        note: None,
        rules: vec![make_file_entry(
            ExprPattern::Exact("group.toml".to_string()),
            None,
        )],
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("resource_dir".to_string()), resource_dir_rule);
    rules.insert(
        RuleName("resource_group_dir".to_string()),
        resource_group_dir_rule,
    );
    rules
}
