use crate::error::LintError;

use super::{dir, leaf, run};

/// Configuration with an optional splice (`- use: rule.tests_rule / optional: true`) that requires a
/// mandatory child directory (`- dir: tests`), whose body in turn requires an exact entry (`- file: mod.rs`).
///
/// The relax should only apply to the body entries directly under the splice (i.e., the `- dir: tests`
/// node-level entries). Once `tests/` materializes and descends into its children, `tests/mod.rs` must
/// remain required (same semantics as the old expand.rs). If the relaxed frame leaks into the child
/// descent, the required check on `tests/mod.rs` is incorrectly skipped (over-relaxation).
///
/// ```text
/// root rule: mycrate :: use crate_like
/// crate_like: src :: <regex file>; (optional) use tests_rule; Cargo.toml
/// tests_rule: tests :: mod.rs   <- mod.rs is a required exact
/// ```
const OPTIONAL_SPLICE_RELAX_YAML: &str = indoc::indoc! {r#"
    version: 0
    entry: root
    rules:
      - rule: root
        ::
          - dir: mycrate
            ::
              - use: rule.crate_like
      - rule: crate_like
        ::
          - dir: src
            ::
              - file:
                  regex: '^[a-z_]+\.rs$'
          - use: rule.tests_rule
            optional: true
          - file: Cargo.toml
      - rule: tests_rule
        ::
          - dir: tests
            ::
              - file: mod.rs
"#};

/// When the optional splice does not materialize (tests/ absent), the required entry directly under
/// the splice (`- dir: tests`) is relaxed and no MissingRequired is reported.
#[test]
fn optional_splice_absent_does_not_report_missing_required() {
    // Arrange: mycrate has src/ and Cargo.toml, but tests/ is absent. The tests_rule splice is optional.
    let mycrate = dir("mycrate", vec![leaf("src", &["lib.rs"])], &["Cargo.toml"]);
    let tree = dir("", vec![mycrate], &[]);

    // Act
    let errors = run(OPTIONAL_SPLICE_RELAX_YAML, &tree);

    // Assert: the required entry directly under the optional splice is relaxed, so no MissingRequired.
    assert!(
        !errors
            .iter()
            .any(|e| matches!(e, LintError::MissingRequired { .. })),
        "required entry under optional splice should be relaxed: {errors:?}"
    );
}

/// When the optional splice materializes (tests/ exists), a missing required exact entry inside it
/// (`tests/mod.rs`) must be reported as MissingRequired (no over-relaxation). Regression test for relaxed frame leaking into child descent.
#[test]
fn optional_splice_materialized_child_still_requires_inner_exact() {
    // Arrange: tests/ exists as a real directory, but the required mod.rs is absent; only some_test.rs is present.
    let mycrate = dir(
        "mycrate",
        vec![leaf("src", &["lib.rs"]), leaf("tests", &["some_test.rs"])],
        &["Cargo.toml"],
    );
    let tree = dir("", vec![mycrate], &[]);

    // Act
    let errors = run(OPTIONAL_SPLICE_RELAX_YAML, &tree);

    // Assert: because tests/ has materialized, tests/mod.rs is required → MissingRequired must be reported.
    let has_mod_rs_missing = errors.iter().any(|e| {
        matches!(
            e,
            LintError::MissingRequired { name, is_dir: false, .. } if name == "mod.rs"
        )
    });
    assert!(
        has_mod_rs_missing,
        "materialized tests/ is missing mod.rs — MissingRequired must be reported (over-relaxation forbidden): {errors:?}"
    );
}
