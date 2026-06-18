use crate::error::SemanticError;
use crate::expr::expr::check::internal::check_with_compat::check_with_compat;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::RuleName;

use super::fixtures::{rule_with_public_id, rule_with_ruletype_input_yaml, splice_entry};
use indexmap::IndexMap;

/// When a RuleType with param is passed an id whose shape matches the rule's public id, no E018.
#[test]
fn rule_type_matching_shape_is_compatible() {
    // Arrange:
    //   feature_dir rule has public id `feat` with capture `stem`
    //   consumer rule declares with x: feature_dir and passes ${feat}
    let feature_dir = rule_with_public_id("feat", r"^(?<stem>[a-z_]+)$");
    let consumer_rule = rule_with_ruletype_input_yaml(
        "x",
        "feature_dir",
        vec![splice_entry("consumer", "x", "${feat}")],
    );
    let mut rules = IndexMap::new();
    rules.insert(RuleName("feature_dir".to_string()), feature_dir);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_compat(&rules, &id_shapes);

    // Assert: no E018
    let e018_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, SemanticError::WithShapeMismatch { .. }))
        .collect();
    assert!(e018_errors.is_empty(), "unexpected E018: {errors:?}");
}
