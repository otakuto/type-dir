use super::{dir, leaf, run};

/// Verifies that a dir with the `/*` skip marker produces no errors even when the directory has contents.
#[test]
fn test_dir_skip_marker_allows_contents() {
    // Arrange: a config with `- dir: foo/*` where foo/ contains arbitrary files.
    let yaml = indoc::indoc! {r#"
        version: 0
        entry: root
        rules:
          - rule: root
            ::
              - dir: foo/*
    "#};
    let tree = dir("", vec![leaf("foo", &["bar.txt", "baz.rs"])], &[]);

    // Act
    let errors = run(yaml, &tree);

    // Assert: no errors.
    assert!(
        errors.is_empty(),
        "dir with skip marker should allow any contents: {errors:?}"
    );
}

/// Verifies that a dir with the `/*` skip marker produces no errors even when the directory has subdirectories.
#[test]
fn test_dir_skip_marker_allows_subdirs() {
    // Arrange: a config with `- dir: foo/*` where foo/ contains subdirectories.
    let yaml = indoc::indoc! {r#"
        version: 0
        entry: root
        rules:
          - rule: root
            ::
              - dir: foo/*
    "#};
    let tree = dir(
        "",
        vec![dir("foo", vec![leaf("sub", &["file.rs"])], &[])],
        &[],
    );

    // Act
    let errors = run(yaml, &tree);

    // Assert: no errors.
    assert!(
        errors.is_empty(),
        "dir with skip marker should allow subdirs: {errors:?}"
    );
}
