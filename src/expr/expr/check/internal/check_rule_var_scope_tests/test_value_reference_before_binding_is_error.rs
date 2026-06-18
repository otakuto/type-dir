use super::super::check_rule_var_scope;
use crate::yaml::{RuleName, ValueExpr, YamlPattern};

use super::fixtures::{bind_entry, dir_entry, rule_with};

/// A `${value.<var>}` reference in a sibling BEFORE the binding is an undeclared ref error
/// (sequential-let: earlier siblings cannot see a later binding).
#[test]
fn value_reference_before_binding_is_error() {
    // Arrange: reference `${value.acc}` first, then bind `acc` afterwards.
    let r = rule_with(
        &[],
        vec![
            dir_entry(YamlPattern::Exact("${value.acc}_dir".to_string())),
            bind_entry("acc", ValueExpr::Scalar("abc".to_string())),
        ],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("r".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: E010 because `acc` is not yet bound at the first sibling.
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    assert!(
        matches!(
            errors[0],
            crate::error::SemanticError::RuleUndeclaredRef { .. }
        ),
        "unexpected: {:?}",
        errors[0]
    );
}
