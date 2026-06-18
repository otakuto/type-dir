use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn ancestor_lookup_falls_back_to_parent_path() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act: look up a path whose leaf (`.dir`) may or may not be in the index,
    // but whose parent entry path is guaranteed to be there.
    let span = index.lookup_with_ancestors("rules.root.rules[0].dir.unknown_leaf");

    // Assert: should fall back to rules.root.rules[0] at the latest.
    assert!(
        span.is_some(),
        "expected ancestor fallback to find rules.root.rules[0]"
    );
}
