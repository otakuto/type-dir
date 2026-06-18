use super::fixtures::*;

/// A path with a bare id segment and no `<kind>` (e.g. `rule.rec_node.node`) is rejected: the path
/// must be a sequence of `<kind>.<name>` pairs (kind ∈ dir/file/regex). An odd number of tail
/// segments cannot form pairs, so it is a parse error.
#[test]
fn bare_path_segment_without_kind_is_error() {
    // Arrange
    let shape = parse("rule.rec_node.node");

    // Act
    let result = shape.to_shape();

    // Assert
    assert!(
        result.is_err(),
        "expected error for a bare path segment without a kind, got: {result:?}"
    );
}
