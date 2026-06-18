use super::{dir, leaf, run};

/// A `value:` list binding is iterated by `for {id: n, value: ${value.names}}`.
const VALUE_LIST_YAML: &str = r#"
version: 0
entry: root
rules:
  - rule: root
    ::
      - id: names
        value: ['a', 'b', 'c']
      - dir: src
        ::
          - for:
              id: n
              value: ${value.names}
            ::
              - file: '${value.n}.rs'
"#;

/// `src` containing exactly the list-derived files (a.rs/b.rs/c.rs) is accepted.
#[test]
fn value_list_drives_for_iteration() {
    // Arrange: src holds the three files named by the value list.
    let src = leaf("src", &["a.rs", "b.rs", "c.rs"]);
    let tree = dir("", vec![src], &[]);

    // Act
    let errors = run(VALUE_LIST_YAML, &tree);

    // Assert: no diagnostics — every file matched a loop iteration.
    assert!(errors.is_empty(), "unexpected: {errors:?}");
}

/// A file outside the value list is reported as undeclared.
#[test]
fn value_list_iteration_rejects_foreign_file() {
    // Arrange: src holds a file (`z.rs`) not produced by the value list.
    let src = leaf("src", &["a.rs", "z.rs"]);
    let tree = dir("", vec![src], &[]);

    // Act
    let errors = run(VALUE_LIST_YAML, &tree);

    // Assert: the foreign file is flagged.
    assert!(!errors.is_empty(), "expected an undeclared diagnostic");
}
