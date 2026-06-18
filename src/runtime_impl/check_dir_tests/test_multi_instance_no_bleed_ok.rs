use super::{MULTI_INSTANCE_YAML, dir, leaf, run};

/// (1) Multi-instance no-bleed — happy path.
/// Each crate's src references only its own crate's q, so the check passes with no excess or deficit.
#[test]
fn test_multi_instance_no_bleed_ok() {
    // Arrange: crate1 and crate2 each have their own consistent queries and src.
    let crate1 = dir(
        "crate1",
        vec![leaf("queries", &["a.sql"]), leaf("src", &["a_sqlx.rs"])],
        &[],
    );
    let crate2 = dir(
        "crate2",
        vec![leaf("queries", &["b.sql"]), leaf("src", &["b_sqlx.rs"])],
        &[],
    );
    let tree = dir("", vec![crate1, crate2], &[]);

    // Act
    let errors = run(MULTI_INSTANCE_YAML, &tree);

    // Assert: no excess or deficit.
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
