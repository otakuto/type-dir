use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_dotted_for_source, file_leaf};

/// `${x.regex.typo}` — capture `typo` is absent from `sub`'s shape — WithShapeMismatch.
#[test]
fn dotted_for_source_binding_to_missing_capture_is_input_shape_mismatch() {
    // Arrange: body references ${x.regex.typo}; typo is not a capture of the sub id
    let body = vec![file_leaf("${x.regex.typo}.txt")];
    let rules = build_rules_dotted_for_source(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: one WithShapeMismatch
    assert_eq!(errors.len(), 1, "expected 1 error: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch, got: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
}
