use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_dotted_for_source, file_leaf};

/// `${x.regex.name}` — capture `name` is in `sub`'s shape — no error.
#[test]
fn dotted_for_source_binding_to_existing_capture_is_ok() {
    // Arrange: body references ${x.regex.name}; name is a capture of the sub id
    let body = vec![file_leaf("${x.regex.name}.txt")];
    let rules = build_rules_dotted_for_source(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
