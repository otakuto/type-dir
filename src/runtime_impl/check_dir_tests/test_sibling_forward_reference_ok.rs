use super::{dir, leaf, run};

/// (3) Sibling forward reference (consumer comes first in lexicographic order).
/// `handler` precedes `schema` lexicographically, but because collection precedes checking,
/// the forward reference via record iteration over `${schema}` resolves correctly.
#[test]
fn test_sibling_forward_reference_ok() {
    // Arrange
    let yaml = r#"
version: 0
entry: root
rules:
  - rule: root
    ::
      - dir: schema
        id: schema_dir
        ::
          - file:
              regex: '^(?<handler>[a-z_]+_(mutation|query))\.rs$'
            id: schema
      - dir: handler
        ::
          - for:
              id: h
              value: ${dir.schema_dir.file.schema}
            ::
              - file: '${value.h.regex.handler}_handler.rs'
"#;
    let schema = leaf("schema", &["foo_mutation.rs"]);
    let handler = leaf("handler", &["foo_mutation_handler.rs"]);
    let tree = dir("", vec![schema, handler], &[]);

    // Act
    let errors = run(yaml, &tree);

    // Assert: forward reference resolves and there is no excess or deficit.
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
