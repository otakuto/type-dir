use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind};
use indexmap::IndexMap;

use super::fixtures::rule_with;

/// `${use.X}` where X is not a self-owned id must produce `RuleUndeclaredRef` (SM008).
#[test]
fn use_ns_unbound_id_is_error() {
    // Arrange: a rule with a splice whose with-arg references ${use.unknown} (not a self-owned id)
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
                m.insert(VarName("nodes".to_string()), "${use.unknown}".to_string());
                m
            },
            colocated_rules: None,
        },
    };
    let r = rule_with(&[], vec![splice_entry]);
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
