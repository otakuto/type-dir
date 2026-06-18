use super::super::check_rule_var_scope;
use crate::yaml::{RuleName, ValueExpr};

use super::fixtures::{bind_entry, rule_with};

/// References inside a value binding's template are validated against the current scope.
/// `${with.p}` is valid when `p` is a declared with-param.
#[test]
fn value_template_with_reference_is_not_an_error() {
    // Arrange: bind `acc` to a template that references the with-param `p`.
    let r = rule_with(
        &["p"],
        vec![bind_entry(
            "acc",
            ValueExpr::Scalar("${with.p}-x".to_string()),
        )],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("r".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {:?}", errors);
}

/// A value binding template referencing a bare name is a BareReference error.
#[test]
fn value_template_undeclared_reference_is_error() {
    // Arrange: bind `acc` to a template referencing an unknown bare name.
    let r = rule_with(
        &[],
        vec![bind_entry(
            "acc",
            ValueExpr::Scalar("${unknown}-x".to_string()),
        )],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("r".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: BareReference because bare references are rejected.
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    assert!(
        matches!(errors[0], crate::error::SemanticError::BareReference { .. }),
        "unexpected: {:?}",
        errors[0]
    );
}
