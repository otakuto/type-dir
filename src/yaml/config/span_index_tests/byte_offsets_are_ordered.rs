use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn byte_offsets_are_ordered() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act
    let root_span = index.lookup_with_ancestors("rules.root").unwrap();
    let entry0_span = index.lookup_with_ancestors("rules.root.rules[0]").unwrap();

    // Assert: the first entry starts after the rule header.
    assert!(
        entry0_span.start >= root_span.start,
        "entry[0] start ({}) should be >= rule start ({})",
        entry0_span.start,
        root_span.start
    );
}
