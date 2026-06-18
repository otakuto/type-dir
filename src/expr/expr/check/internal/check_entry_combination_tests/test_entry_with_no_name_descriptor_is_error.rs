use super::super::check_entry_combination;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

#[test]
fn entry_with_no_name_descriptor_is_error() {
    // Arrange: entry with no dir/file/rule/group/for — the Group{rules:vec![]} sentinel
    let entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Group {
            body: vec![],
            explicit_marker: false,
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
