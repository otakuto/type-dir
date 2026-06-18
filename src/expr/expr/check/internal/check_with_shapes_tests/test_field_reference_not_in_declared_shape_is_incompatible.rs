use super::super::check_with_shapes;
use crate::error::SemanticError;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::RuleName;

use super::fixtures::{dir_id_entry, file_leaf, for_items_entry, rule_with_ruletype_input};

/// Kind-qualified reference `${x.regex.missing}` absent from the RuleType-derived id shape is WithShapeMismatch (E018).
///
/// (Previously tested with a Records shape; now uses RuleType since Records is removed.)
#[test]
fn field_reference_not_in_declared_shape_is_incompatible() {
    // Arrange: producer has public id `it` (dir) with capture `stem`
    //          consumer declares `items: producer` (RuleType) and iterates with for x in ${choice.items}
    //          body references the non-existent ${x.regex.missing}
    let producer_entry = dir_id_entry(r"^(?<stem>[a-z]+)$", "it", vec![]);
    let producer_rule = crate::yaml::YamlRule {
        rule: crate::yaml::RuleName("producer".to_string()),
        with_params: indexmap::IndexMap::new(),
        note: None,
        body: vec![producer_entry],
    };
    let body = vec![file_leaf("${x.regex.missing}_sqlx.rs")];
    let consumer_rule = rule_with_ruletype_input("items", "producer", vec![for_items_entry(body)]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("producer".to_string()), producer_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_shapes(&rules, &id_shapes);

    // Assert: one WithShapeMismatch
    assert_eq!(errors.len(), 1, "expected 1 WithShapeMismatch: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, with, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(with, "items");
}
