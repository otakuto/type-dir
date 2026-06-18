use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_for_binding_dir_file_kind, file_leaf};

/// Reference `${x.dir.leaf}` requests dir but leaf is a file — NodeKindMismatch (E022).
#[test]
fn dir_qualified_reference_to_file_child_is_node_kind_mismatch() {
    // Arrange: body uses ${x.dir.leaf}; leaf is a file child, so .dir. is wrong
    let body = vec![file_leaf("${x.dir.leaf}")];
    let rules = build_rules_for_binding_dir_file_kind(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: exactly one NodeKindMismatch with expected="dir", actual="file"
    assert_eq!(errors.len(), 1, "expected 1 NodeKindMismatch: {errors:?}");
    let SemanticError::NodeKindMismatch {
        rule,
        reference,
        expected,
        actual,
    } = &errors[0]
    else {
        panic!("expected NodeKindMismatch: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(reference, "x.dir.leaf");
    assert_eq!(expected, "dir");
    assert_eq!(actual, "file");
}
