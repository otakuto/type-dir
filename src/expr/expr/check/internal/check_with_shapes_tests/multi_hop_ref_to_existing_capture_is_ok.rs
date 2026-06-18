use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_multi_hop_ref, file_leaf};

/// `${f.dir.a.regex.g}` — `a` is a child dir and `g` is its capture — no error.
#[test]
fn multi_hop_ref_to_existing_capture_is_ok() {
    // Arrange: consumer body uses ${f.dir.a.regex.g}; a is a child dir of it; g is a capture of a
    let body = vec![file_leaf("${f.dir.a.regex.g}.txt")];
    let rules = build_rules_multi_hop_ref(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
