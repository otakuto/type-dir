use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_for_items, file_leaf};

/// Kind-qualified reference `${x.regex.missing}` absent from the derived RuleType id shape is WithShapeMismatch (E018).
#[test]
fn for_binding_ruletype_missing_field_is_incompatible() {
    // Arrange: body uses ${x.regex.missing} which is not in the derived IdShape captures
    let body = vec![file_leaf("${x.regex.missing}.txt")];
    let rules = build_rules_for_items(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: one WithShapeMismatch for the consumer rule with with="items"
    assert_eq!(errors.len(), 1, "expected 1 WithShapeMismatch: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, with, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(with, "items");
}
