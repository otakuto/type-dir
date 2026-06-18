use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_for_items, file_leaf};

/// Kind-qualified reference `${x.regex.stem}` present in the derived RuleType id shape produces no error.
#[test]
fn for_binding_ruletype_valid_field_is_compatible() {
    // Arrange: body uses ${x.regex.stem} which is in the derived IdShape captures
    let body = vec![file_leaf("${x.regex.stem}.txt")];
    let rules = build_rules_for_items(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
