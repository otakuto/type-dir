use crate::expr::expr::check::internal::check_with_compat::check_with_compat;
use crate::expr::expr::check::internal::id_shape_derive::build_id_shapes;
use crate::yaml::RuleName;

use super::fixtures::{empty_rule, id_file_entry, rule_with_ruletype_input, splice_entry};

/// When a RuleType input is passed an id whose shape subsumes the declared rule shape, no E018.
///
/// (Previously tested with Records shape; now uses RuleType since Records is removed.)
#[test]
fn declared_field_in_captures_is_compatible() {
    // Arrange: producer has id `q` with capture `stem`; consumer declares `q: producer` (RuleType)
    //          and passes ${q} — shape subsumes the producer's public id shape (compatible).
    let producer = id_file_entry("q", r"^(?<stem>.+)\.sql$");
    let producer_rule = empty_rule(vec![producer]);
    let consumer_rule =
        rule_with_ruletype_input("q", "producer", vec![splice_entry("consumer", "q", "${q}")]);

    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("producer".to_string()), producer_rule);
    rules.insert(RuleName("consumer".to_string()), consumer_rule);

    // Act
    let id_shapes = build_id_shapes(&rules);
    let errors = check_with_compat(&rules, &id_shapes);

    // Assert
    assert!(errors.is_empty(), "unexpected error: {errors:?}");
}
