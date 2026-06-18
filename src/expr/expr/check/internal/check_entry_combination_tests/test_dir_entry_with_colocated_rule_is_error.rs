use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::make_rule;

#[test]
fn dir_entry_with_colocated_rule_is_error() {
    // Arrange: dir entry with a colocated rule (as produced by From<PlainEntry> when both dir and rule are set)
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("foo".to_string()),
            body: None,
            colocated_use_ref: Some(RuleName("cs".to_string())),
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert
    assert_eq!(errors.len(), 1);
    assert!(matches!(&errors[0], SemanticError::DirFileWithRule { .. }));
}
