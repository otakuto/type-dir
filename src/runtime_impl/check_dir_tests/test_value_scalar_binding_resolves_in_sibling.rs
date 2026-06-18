use super::{dir, leaf, run};

/// A `value:` scalar binding is visible to a later sibling via `${value.<var>}`.
const VALUE_SCALAR_YAML: &str = r#"
version: 0
entry: root
rules:
  - rule: root
    ::
      - id: acc
        value: 'fixed'
      - dir: '${value.acc}_dir'
        ::
          - file: keep.txt
"#;

/// A directory named after the scalar value binding is accepted; a foreign name is reported.
#[test]
fn value_scalar_binding_resolves_in_sibling() {
    // Arrange: the tree contains `fixed_dir` (matches `${value.acc}_dir`).
    let fixed_dir = leaf("fixed_dir", &["keep.txt"]);
    let tree = dir("", vec![fixed_dir], &[]);

    // Act
    let errors = run(VALUE_SCALAR_YAML, &tree);

    // Assert: no diagnostics — the value-bound name resolved and matched.
    assert!(errors.is_empty(), "unexpected: {errors:?}");
}

/// A directory not matching the resolved value binding is reported as undeclared.
#[test]
fn value_scalar_binding_rejects_foreign_dir() {
    // Arrange: the tree contains `other_dir` instead of `fixed_dir`.
    let other_dir = leaf("other_dir", &["keep.txt"]);
    let tree = dir("", vec![other_dir], &[]);

    // Act
    let errors = run(VALUE_SCALAR_YAML, &tree);

    // Assert: the unexpected directory is flagged.
    assert!(!errors.is_empty(), "expected an undeclared diagnostic");
}
