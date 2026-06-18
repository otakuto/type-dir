use super::super::build_span_index;

#[test]
fn invalid_yaml_returns_empty_index() {
    // Arrange
    let bad_yaml = "{{{{not valid yaml}}}}}";

    // Act
    let index = build_span_index(bad_yaml);

    // Assert: no panic, index is empty / degrade-safe
    let span = index.lookup_with_ancestors("rules.foo");
    assert!(span.is_none());
}
