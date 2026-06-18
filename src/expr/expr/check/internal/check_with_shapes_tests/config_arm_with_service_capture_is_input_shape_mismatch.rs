use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{arm_entry, build_rules_match_arm_sum_narrowing, file_leaf};

/// Inside the `config` arm, `${c.regex.svc}` references the capture of the `service` alternative
/// — must produce `WithShapeMismatch` (E018).
#[test]
fn config_arm_with_service_capture_is_input_shape_mismatch() {
    // Arrange: config arm incorrectly uses ${c.regex.svc} (svc is a service-alt capture, not config)
    let service_arm = arm_entry("service", vec![file_leaf("${c.regex.svc}.md")]);
    let config_arm = arm_entry("config", vec![file_leaf("${c.regex.svc}.md")]);
    let rules = build_rules_match_arm_sum_narrowing(vec![service_arm, config_arm]);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: exactly one WithShapeMismatch (in the config arm)
    assert_eq!(errors.len(), 1, "expected 1 WithShapeMismatch: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch, got: {:?}", errors[0]);
    };
    assert_eq!(rule, "root");
}
