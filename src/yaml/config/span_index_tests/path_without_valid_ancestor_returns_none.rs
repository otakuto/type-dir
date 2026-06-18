use super::super::build_span_index;
use super::fixtures::SAMPLE_YAML;

#[test]
fn path_without_valid_ancestor_returns_none() {
    // Arrange
    let index = build_span_index(SAMPLE_YAML);

    // Act: a dot-less path has no valid ancestor prefix to fall back to.
    let span = index.lookup_with_ancestors("nonexistent");

    // Assert
    assert!(
        span.is_none(),
        "expected a path with no valid ancestor to return None"
    );
}
