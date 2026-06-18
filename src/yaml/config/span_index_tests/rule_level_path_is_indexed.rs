use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn rule_level_path_is_indexed() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act
    let span = index.lookup_with_ancestors("rules.root");

    // Assert
    assert!(span.is_some(), "expected rules.root to be indexed");
}
