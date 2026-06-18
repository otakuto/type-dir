use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn second_entry_level_path_is_indexed() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act
    let span = index.lookup_with_ancestors("rules.root.rules[1]");

    // Assert
    assert!(span.is_some(), "expected rules.root.rules[1] to be indexed");
}
