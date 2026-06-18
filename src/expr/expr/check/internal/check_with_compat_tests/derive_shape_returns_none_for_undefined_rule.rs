use crate::expr::expr::check::internal::id_shape_derive::derive_rule_id_shape;
use crate::yaml::RuleName;

/// For an undefined rule, derive_rule_id_shape returns None.
#[test]
fn derive_shape_returns_none_for_undefined_rule() {
    // Arrange: empty rule map
    let rules = indexmap::IndexMap::new();

    // Act
    let shape = derive_rule_id_shape(&RuleName("nonexistent".to_string()), &rules);

    // Assert
    assert!(
        shape.is_none(),
        "expected None for undefined rule, got: {shape:?}"
    );
}
