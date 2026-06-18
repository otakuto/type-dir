use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_multi_hop_ref, file_leaf};

/// `${f.dir.a.regex.missing}` — `missing` is not a capture of `a` — WithShapeMismatch.
#[test]
fn multi_hop_ref_to_missing_capture_is_input_shape_mismatch() {
    // Arrange: consumer body uses ${f.dir.a.regex.missing}; missing is not a capture of a
    let body = vec![file_leaf("${f.dir.a.regex.missing}.txt")];
    let rules = build_rules_multi_hop_ref(body);

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
