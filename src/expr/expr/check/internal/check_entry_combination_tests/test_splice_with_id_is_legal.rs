use super::super::check_entry_combination;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::make_rule;

#[test]
fn splice_with_id_is_legal() {
    // Arrange: bare rule with colocated id (α-rename; allowed since spec phase 1)
    let entry = YamlEntry {
        id: Some(EntryId("x".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName("cs".to_string()),
            with_args: IndexMap::new(),
            colocated_rules: None,
        },
    };
    let mut rules = IndexMap::new();
    rules.insert(RuleName("parent_rule".to_string()), make_rule(vec![entry]));

    // Act
    let errors = check_entry_combination(&rules);

    // Assert: splice+id must not produce SpliceWithSubtree
    assert!(
        !errors
            .iter()
            .any(|e| matches!(e, crate::error::SemanticError::SpliceWithSubtree { .. })),
        "splice+id should be legal but got SpliceWithSubtree errors: {:?}",
        errors
    );
}
