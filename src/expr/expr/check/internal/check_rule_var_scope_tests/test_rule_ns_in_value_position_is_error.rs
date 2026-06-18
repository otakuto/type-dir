use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{EntryId, RuleName, VarName, YamlEntry, YamlEntryKind, YamlPattern};
use indexmap::IndexMap;

use super::fixtures::rule_with;

/// `${rule.X}` used in a value position (with-arg) must produce `RuleNsInValuePosition` (SM020).
/// `rule.` is a type namespace; use `${use.<id>}` to reference a splice instance value.
#[test]
fn rule_ns_in_value_position_is_error() {
    // Arrange: a rule with a splice whose with-arg contains ${rule.rec} in a value position
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
                m.insert(
                    VarName("nodes".to_string()),
                    "${rule.rec.dir.node}".to_string(),
                );
                m
            },
            colocated_rules: None,
        },
    };
    // Add a dir entry with id: rec so that rec is a self-owned id
    let rec_dir = YamlEntry {
        id: Some(EntryId("rec".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("rec".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let r = rule_with(&[], vec![rec_dir, splice_entry]);
    let mut rules = IndexMap::new();
    rules.insert(RuleName("root".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, SemanticError::RuleNsInValuePosition { .. })),
        "expected RuleNsInValuePosition, got: {:?}",
        errors
    );
}
