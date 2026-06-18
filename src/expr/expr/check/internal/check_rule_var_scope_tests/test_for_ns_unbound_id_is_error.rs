use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::rule_with;

/// `${for.unknown}` where `unknown` is not a self-owned for-id must produce `RuleUndeclaredRef`.
#[test]
fn for_ns_unbound_id_is_error() {
    // Arrange: a splice that references ${for.unknown} (no for entry with id: unknown in scope)
    let splice_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName("flatten".to_string()),
            with_args: {
                let mut m = IndexMap::new();
                m.insert(VarName("nodes".to_string()), "${for.unknown}".to_string());
                m
            },
            colocated_rules: None,
        },
    };
    // A plain dir entry (not a for+id entry) to confirm it does not provide the id
    let dir_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("src".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let r = rule_with(&[], vec![dir_entry, splice_entry]);
    let mut rules = IndexMap::new();
    rules.insert(RuleName("root".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, SemanticError::RuleUndeclaredRef { .. })),
        "expected RuleUndeclaredRef, got: {:?}",
        errors
    );
}
