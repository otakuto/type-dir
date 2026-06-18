use crate::error::LintError;

use super::{MULTI_INSTANCE_YAML, dir, leaf, run};

/// (2) Multi-instance no-bleed — contamination detection.
/// crate2/src contains another crate's file a_sqlx.rs while its own b_sqlx.rs is missing.
/// If q is scoped to {b} in crate2 and a does not bleed through, a_sqlx.rs becomes Undeclared.
#[test]
fn test_multi_instance_no_bleed_detects_foreign() {
    // Arrange: place foreign a_sqlx.rs in crate2/src (b_sqlx.rs is absent).
    let crate1 = dir(
        "crate1",
        vec![leaf("queries", &["a.sql"]), leaf("src", &["a_sqlx.rs"])],
        &[],
    );
    let crate2 = dir(
        "crate2",
        vec![leaf("queries", &["b.sql"]), leaf("src", &["a_sqlx.rs"])],
        &[],
    );
    let tree = dir("", vec![crate1, crate2], &[]);

    // Act
    let errors = run(MULTI_INSTANCE_YAML, &tree);

    // Assert: since there is no contamination, a_sqlx.rs becomes Undeclared.
    assert!(
        !errors.is_empty(),
        "contamination detection error not raised"
    );
    let has_foreign_undeclared = errors.iter().any(|e| match e {
        LintError::Undeclared { path, .. } => path.to_string_lossy().contains("a_sqlx.rs"),
        _ => false,
    });
    assert!(
        has_foreign_undeclared,
        "Undeclared for crate2/src/a_sqlx.rs not found: {errors:?}"
    );
    // Supplementary: MissingRequired for b_sqlx.rs should also be present.
    let has_missing_b = errors.iter().any(|e| match e {
        LintError::MissingRequired { name, .. } => name.contains("b_sqlx.rs"),
        _ => false,
    });
    assert!(
        has_missing_b,
        "MissingRequired for crate2's b_sqlx.rs not found: {errors:?}"
    );
}
