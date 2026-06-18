use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_for_items, file_leaf};

/// Reference `${x.regex.nope}` — capture `nope` is not in the id shape — WithShapeMismatch (E018).
#[test]
fn regex_qualified_reference_to_unknown_capture_is_incompatible() {
    // Arrange: body uses ${x.regex.nope}; nope is not a capture in the id shape of it
    let body = vec![file_leaf("${x.regex.nope}.txt")];
    let rules = build_rules_for_items(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: exactly one WithShapeMismatch
    assert_eq!(errors.len(), 1, "expected 1 WithShapeMismatch: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, with, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(with, "items");
}
