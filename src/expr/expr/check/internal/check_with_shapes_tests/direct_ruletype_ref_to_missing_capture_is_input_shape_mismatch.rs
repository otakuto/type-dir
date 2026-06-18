use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_direct_ruletype_input_ref, file_leaf};

/// Direct reference `${f.regex.typo}` — capture `typo` is absent — WithShapeMismatch.
#[test]
fn direct_ruletype_ref_to_missing_capture_is_input_shape_mismatch() {
    // Arrange: consumer body uses ${f.regex.typo} directly; typo is not a capture in producer
    let body = vec![file_leaf("${f.regex.typo}.txt")];
    let rules = build_rules_direct_ruletype_input_ref(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: one WithShapeMismatch
    assert_eq!(errors.len(), 1, "expected 1 error: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, with, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch, got: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(with, "f");
}
