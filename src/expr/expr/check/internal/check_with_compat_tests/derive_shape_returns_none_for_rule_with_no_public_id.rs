use crate::expr::expr::check::internal::id_shape_derive::derive_rule_id_shape;
use crate::yaml::RuleName;

use super::fixtures::empty_rule;

/// For a rule with no public id, derive_rule_id_shape returns None.
#[test]
fn derive_shape_returns_none_for_rule_with_no_public_id() {
    // Arrange: rule with no id-bearing entries
    let rule = empty_rule(vec![]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("empty_rule".to_string()), rule);

    // Act
    let shape = derive_rule_id_shape(&RuleName("empty_rule".to_string()), &rules);

    // Assert
    assert!(
        shape.is_none(),
        "expected None for rule with no public id, got: {shape:?}"
    );
}
