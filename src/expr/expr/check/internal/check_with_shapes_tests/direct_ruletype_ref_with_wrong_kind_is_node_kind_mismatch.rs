use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{build_rules_direct_ruletype_input_ref, file_leaf};

/// Direct reference `${f.dir.leaf}` — `leaf` is a file child, not dir — NodeKindMismatch.
#[test]
fn direct_ruletype_ref_with_wrong_kind_is_node_kind_mismatch() {
    // Arrange: consumer body uses ${f.dir.leaf}; leaf is a file, not a dir
    let body = vec![file_leaf("${f.dir.leaf}")];
    let rules = build_rules_direct_ruletype_input_ref(body);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: one NodeKindMismatch
    assert_eq!(errors.len(), 1, "expected 1 error: {errors:?}");
    let SemanticError::NodeKindMismatch {
        rule,
        reference,
        expected,
        actual,
    } = &errors[0]
    else {
        panic!("expected NodeKindMismatch, got: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(reference, "f.dir.leaf");
    assert_eq!(expected, "dir");
    assert_eq!(actual, "file");
}
