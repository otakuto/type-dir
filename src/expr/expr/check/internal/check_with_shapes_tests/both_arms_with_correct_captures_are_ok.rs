use super::super::check_with_shapes;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;

use super::fixtures::{arm_entry, build_rules_match_arm_sum_narrowing, file_leaf};

/// Both arms use their respective correct captures — no error.
#[test]
fn both_arms_with_correct_captures_are_ok() {
    // Arrange: service arm uses ${c.regex.svc}, config arm uses ${c.regex.cfg}
    let service_arm = arm_entry("service", vec![file_leaf("${c.regex.svc}.md")]);
    let config_arm = arm_entry("config", vec![file_leaf("${c.regex.cfg}.md")]);
    let rules = build_rules_match_arm_sum_narrowing(vec![service_arm, config_arm]);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: no errors
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
