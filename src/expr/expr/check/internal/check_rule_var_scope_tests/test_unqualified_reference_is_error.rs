use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlPattern};

use super::fixtures::{dir_entry, rule_with};

/// Dotted reference `${gql_op.op}` with a bare head is rejected as BareReference before the tail is inspected.
#[test]
fn unqualified_reference_is_error() {
    // Arrange
    let r = rule_with(
        &[],
        vec![dir_entry(YamlPattern::Exact(
            "${gql_op.op}_handler".to_string(),
        ))],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("handler".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert_eq!(errors.len(), 1);
    let SemanticError::BareReference { reference, .. } = &errors[0] else {
        panic!("unexpected: {:?}", errors[0]);
    };
    assert_eq!(reference, "gql_op.op");
}
