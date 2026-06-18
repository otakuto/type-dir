use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_direct_ruletype_input_ref, file_leaf};

/// Direct reference `${f.regex.stem}` — capture `stem` is in the id shape — no error.
#[test]
fn direct_ruletype_ref_to_existing_capture_is_ok() {
    // Arrange: consumer body uses ${f.regex.stem} directly (no for loop)
    let body = vec![file_leaf("${f.regex.stem}.txt")];
    let rules = build_rules_direct_ruletype_input_ref(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
