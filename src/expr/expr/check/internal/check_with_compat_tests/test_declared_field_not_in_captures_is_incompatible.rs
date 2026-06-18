use crate::error::SemanticError;
use crate::expr::expr::check::internal::check_with_compat::check_with_compat;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::RuleName;

use super::fixtures::{empty_rule, id_file_entry, rule_with_ruletype_input, splice_entry};

/// When a RuleType with param is passed an id whose shape does not subsume the declared rule shape, E018.
///
/// Here `feature_dir` has capture `stem`, but we pass `q` (which only has `stem` too in this
/// fixture — the key point is the declared type is `feature_dir` and the passed id `q` has no
/// such association, tested via subsumption).
///
/// (Previously tested with Records shape; now uses RuleType since Records is removed.)
///
/// Concretely: `feature_dir` rule has public id `feat` with capture `stem`.
/// `q_rule` rule has public id `q` with capture `name` (different capture set).
/// Consumer declares `q: feature_dir` and passes `${q}` — mismatch because `q` lacks `stem`.
#[test]
fn declared_field_not_in_captures_is_incompatible() {
    // Arrange: feature_dir rule → public id `feat` with capture `stem`
    //          q_rule → public id `q` with capture `name` (no `stem`)
    //          consumer declares `q: feature_dir` and passes ${q} — incompatible
    let feat_entry = id_file_entry("feat", r"^(?<stem>.+)\.sql$");
    let feature_dir_rule = empty_rule(vec![feat_entry]);

    let q_entry = id_file_entry("q", r"^(?<name>.+)\.sql$");
    let q_rule = empty_rule(vec![q_entry]);

    let consumer_rule = rule_with_ruletype_input(
        "q",
        "feature_dir",
        vec![splice_entry("consumer", "q", "${file.q}")],
    );

    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("feature_dir".to_string()), feature_dir_rule);
    rules.insert(RuleName("q_rule".to_string()), q_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_compat(&rules, &id_shapes);

    // Assert: one WithShapeMismatch (q lacks the `stem` capture required by feature_dir)
    assert_eq!(errors.len(), 1, "expected 1 WithShapeMismatch: {errors:?}");
    let SemanticError::WithShapeMismatch { rule, with, .. } = &errors[0] else {
        panic!("expected WithShapeMismatch: {:?}", errors[0]);
    };
    assert_eq!(rule, "consumer");
    assert_eq!(with, "q");
}
