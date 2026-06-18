use super::super::check_rule_var_scope;
use crate::yaml::{RuleName, ValueExpr, YamlPattern};

use super::fixtures::{bind_entry, dir_entry, rule_with};

/// A `${value.<var>}` reference in a sibling after the binding is in scope (sequential-let).
#[test]
fn value_namespace_reference_is_not_an_error() {
    // Arrange: bind `acc`, then reference it via `${value.acc}` in a later sibling.
    let r = rule_with(
        &[],
        vec![
            bind_entry("acc", ValueExpr::Scalar("abc".to_string())),
            dir_entry(YamlPattern::Exact("${value.acc}_dir".to_string())),
        ],
    );
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("r".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {:?}", errors);
}
