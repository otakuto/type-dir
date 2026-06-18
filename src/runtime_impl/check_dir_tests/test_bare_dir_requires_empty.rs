use crate::error::LintError;

use super::{dir, leaf, run};

/// Verifies that a bare dir (no `::`) requires the directory to be empty.
/// When `foo/` contains any file, an Undeclared (LT001) error is produced;
/// when `foo/` is empty, validation passes.
#[test]
fn test_bare_dir_with_contents_is_undeclared() {
    // Arrange: a config with `- dir: foo` (bare dir) where foo/ contains a file.
    let yaml = indoc::indoc! {r#"
        version: 0
        entry: root
        rules:
          - rule: root
            ::
              - dir: foo
    "#};
    let tree = dir("", vec![leaf("foo", &["bar.txt"])], &[]);

    // Act
    let errors = run(yaml, &tree);

    // Assert: bar.txt becomes Undeclared.
    assert!(
        !errors.is_empty(),
        "bare dir with contents should produce errors"
    );
    let has_undeclared = errors.iter().any(|e| match e {
        LintError::Undeclared { path, .. } => path.to_string_lossy().contains("bar.txt"),
        _ => false,
    });
    assert!(
        has_undeclared,
        "Undeclared for foo/bar.txt not found: {errors:?}"
    );
}

/// Verifies that a bare dir (no `::`) with an empty directory produces no errors.
#[test]
fn test_bare_dir_empty_is_ok() {
    // Arrange: a config with `- dir: foo` (bare dir) where foo/ is empty.
    let yaml = indoc::indoc! {r#"
        version: 0
        entry: root
        rules:
          - rule: root
            ::
              - dir: foo
    "#};
    let tree = dir("", vec![leaf("foo", &[])], &[]);

    // Act
    let errors = run(yaml, &tree);

    // Assert: no errors.
    assert!(
        errors.is_empty(),
        "empty bare dir should have no errors: {errors:?}"
    );
}
