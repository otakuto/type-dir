use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_for_items, file_leaf};

/// Reference `${x.regex.stem}` — capture `stem` is in the id shape — no error.
#[test]
fn regex_qualified_reference_to_known_capture_is_ok() {
    // Arrange: body uses ${x.regex.stem}; stem is a capture in the id shape of it
    let body = vec![file_leaf("${x.regex.stem}.txt")];
    let rules = build_rules_for_items(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
