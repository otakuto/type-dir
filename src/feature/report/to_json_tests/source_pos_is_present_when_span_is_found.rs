use crate::error::SemanticError;

use crate::feature::report::compile_errors_to_json;

#[test]
fn source_pos_is_present_when_span_is_found() {
    // Arrange: build a span index from a minimal YAML that contains a rule with an entry.
    let yaml = "version: 0\nentry: root\nrules:\n  - rule: root\n    ::\n      - use: rule.child\n";
    let span_index = crate::yaml::build_span_index(yaml);
    let error = SemanticError::DirFileWithRule {
        context: "rules.root.rules[0]".to_string(),
    };

    // Act
    let value = compile_errors_to_json(std::slice::from_ref(&error), &span_index);

    // Assert
    let pos = &value["errors"][0]["source_pos"];
    assert!(
        !pos.is_null(),
        "source_pos should be present when span is found"
    );
    assert!(pos["start"].as_u64().is_some(), "start should be a number");
    assert!(pos["end"].as_u64().is_some(), "end should be a number");
}
