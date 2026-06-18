use super::{dir, leaf, run};

/// A `value:` binding interpolates `${with.<param>}` against the rule's with scope, and the result
/// is usable through `${value.<var>}` in a sibling.
const VALUE_INTERP_YAML: &str = indoc::indoc! {r#"
    version: 0
    entry: root
    rules:
      - rule: root
        ::
          - dir: feature
            ::
              - use: rule.feature_dir
                with:
                  - id: p
                    value: api
      - rule: feature_dir
        with:
          - id: p
            type: type.string
        ::
          - id: acc
            value: '${with.p}-handler'
          - file: '${value.acc}.rs'
"#};

/// `api-handler.rs` is accepted because `acc` resolves to `api-handler`.
#[test]
fn value_binding_interpolates_with_param() {
    // Arrange: feature/api-handler.rs matches `${value.acc}.rs` with acc = `${with.p}-handler`.
    let feature = leaf("feature", &["api-handler.rs"]);
    let tree = dir("", vec![feature], &[]);

    // Act
    let errors = run(VALUE_INTERP_YAML, &tree);

    // Assert: no diagnostics — the interpolated value resolved and matched.
    assert!(errors.is_empty(), "unexpected: {errors:?}");
}

/// A file not matching the interpolated value is reported.
#[test]
fn value_binding_interpolation_rejects_foreign_file() {
    // Arrange: feature holds a file that does not match the interpolated name.
    let feature = leaf("feature", &["wrong.rs"]);
    let tree = dir("", vec![feature], &[]);

    // Act
    let errors = run(VALUE_INTERP_YAML, &tree);

    // Assert: the foreign file is flagged.
    assert!(!errors.is_empty(), "expected an undeclared diagnostic");
}
