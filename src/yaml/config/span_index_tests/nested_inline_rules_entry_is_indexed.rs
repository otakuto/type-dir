use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn nested_inline_rules_entry_is_indexed() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act
    let span = index.lookup_with_ancestors("rules.root.rules[0].rules[0]");

    // Assert
    assert!(
        span.is_some(),
        "expected rules.root.rules[0].rules[0] to be indexed"
    );
}
