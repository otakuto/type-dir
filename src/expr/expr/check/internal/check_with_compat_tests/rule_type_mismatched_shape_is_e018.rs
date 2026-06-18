use crate::error::SemanticError;
use crate::expr::expr::check::internal::check_with_compat::check_with_compat;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::{EntryId, PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern};

use super::fixtures::{
    empty_rule, rule_with_public_id, rule_with_ruletype_input_yaml, splice_entry,
};
use indexmap::IndexMap;

/// When a RuleType input is passed an id whose shape is incompatible (missing capture), E018.
#[test]
fn rule_type_mismatched_shape_is_e018() {
    // Arrange:
    //   feature_dir rule has public id `feat` with capture `stem`
    //   plain_dir rule has public id `plain` with NO captures (plain name entry)
    //   consumer rule declares with x: feature_dir and passes ${plain}
    let feature_dir = rule_with_public_id("feat", r"^(?<stem>[a-z_]+)$");

    // plain_dir: id-bearing entry with spec pattern and no regex (no captures)
    let plain_entry = YamlEntry {
        id: Some(EntryId("plain".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Spec(PatternSpec { regex: None }),
            body: None,
            colocated_use_ref: None,
        },
    };
    let plain_dir = empty_rule(vec![plain_entry]);

    let consumer_rule = rule_with_ruletype_input_yaml(
        "x",
        "feature_dir",
        vec![splice_entry("consumer", "x", "${dir.plain}")],
    );
    let mut rules = IndexMap::new();
    rules.insert(RuleName("feature_dir".to_string()), feature_dir);
    rules.insert(RuleName("plain_dir".to_string()), plain_dir);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_compat(&rules, &id_shapes);

    // Assert: E018 emitted because `plain` lacks the `stem` capture
    let e018_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, SemanticError::WithShapeMismatch { .. }))
        .collect();
    assert_eq!(e018_errors.len(), 1, "expected 1 E018: {errors:?}");
    match e018_errors[0] {
        SemanticError::WithShapeMismatch { rule, with, .. } => {
            assert_eq!(rule.as_str(), "consumer");
            assert_eq!(with.as_str(), "x");
        }
        _ => unreachable!(),
    }
}
