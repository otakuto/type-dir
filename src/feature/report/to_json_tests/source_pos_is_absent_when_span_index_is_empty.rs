use crate::error::SemanticError;
use crate::yaml::SpanIndex;

use crate::feature::report::compile_errors_to_json;

#[test]
fn source_pos_is_absent_when_span_index_is_empty() {
    // Arrange
    let error = SemanticError::DirFileWithRule {
        context: "rules.foo.rules[0]".to_string(),
    };
    let span_index = SpanIndex::default();

    // Act
    let value = compile_errors_to_json(std::slice::from_ref(&error), &span_index);

    // Assert: source_pos should not be present when the index is empty
    assert!(
        value["errors"][0]["source_pos"].is_null(),
        "source_pos should be absent for empty index"
    );
}
