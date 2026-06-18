use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn unknown_rule_falls_back_to_rules_block() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act: a nonexistent rule path has no exact entry, so the ancestor walk
    // strips `.nonexistent` and lands on the `rules` block (which is indexed).
    let span = index.lookup_with_ancestors("rules.nonexistent");
    let rules_block = index.lookup_with_ancestors("rules");

    // Assert: degrades gracefully to the `rules:` block span.
    assert!(
        span.is_some(),
        "expected nonexistent rule path to degrade to the rules block"
    );
    assert_eq!(
        span, rules_block,
        "fallback span should point at the rules: block"
    );
}
