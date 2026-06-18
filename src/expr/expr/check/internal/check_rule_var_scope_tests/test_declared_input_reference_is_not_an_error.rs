use super::super::check_rule_var_scope;
use crate::yaml::{RuleName, YamlPattern};

use super::fixtures::{dir_entry, rule_with};

/// References to with-params via `${with.<param>}` are allowed.
#[test]
fn declared_input_reference_is_not_an_error() {
    // Arrange: use `${with.op}` (with-namespace prefix required for with-params)
    let r = rule_with(
        &["op"],
        vec![dir_entry(YamlPattern::Exact(
            "${with.op}_handler".to_string(),
        ))],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("handler".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {:?}", errors);
}

/// Bare reference to a with-param (`${op}` without `with.` prefix) is a BareReference error.
#[test]
fn bare_reference_to_with_param_is_error() {
    // Arrange: use bare `${op}` (no `with.` prefix) — should be rejected
    let r = rule_with(
        &["op"],
        vec![dir_entry(YamlPattern::Exact("${op}_handler".to_string()))],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("handler".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: BareReference because bare references are rejected regardless of declared with-params
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    assert!(
        matches!(errors[0], crate::error::SemanticError::BareReference { .. }),
        "unexpected: {:?}",
        errors[0]
    );
}
