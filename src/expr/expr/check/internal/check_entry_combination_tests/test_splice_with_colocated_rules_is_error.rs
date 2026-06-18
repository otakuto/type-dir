use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

#[test]
fn splice_with_colocated_rules_is_error() {
    // Arrange: bare rule with colocated inline rules (as produced by From<PlainEntry> when both rule and rules are set)
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName("cs".to_string()),
            with_args: IndexMap::new(),
            colocated_rules: Some(vec![]),
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, SemanticError::SpliceWithSubtree { .. }))
    );
}
