use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_for_binding_dir_file_kind, file_leaf};

/// Reference `${x.file.leaf}` matches the actual kind (leaf is a file) — no error.
#[test]
fn file_qualified_reference_to_file_child_is_ok() {
    // Arrange: body uses ${x.file.leaf}; leaf is a file child of it
    let body = vec![file_leaf("${x.file.leaf}")];
    let rules = build_rules_for_binding_dir_file_kind(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
