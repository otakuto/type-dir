use super::{dir, leaf, run};

/// Self-owned id collection across a splice: a configuration where an id-less dir → file id
/// inside the splice-target rule is consumed by a for in the same rule resolves even when called
/// via a splice from the caller node (on-site collection).
///
/// Minimal characterization that owned ids are supplied at the rule's own position even after
/// removing Γ_set pass-through.
#[test]
fn test_splice_supplies_own_id_on_site() {
    // Arrange: root splices a single crate_dir. Inside crate_dir, queries (no id) / q (file id)
    // are collected, and consumed by a for inside src via ${x.regex.name}.
    let yaml = indoc::indoc! {r#"
        version: 0
        entry: root
        rules:
          - rule: root
            ::
              - use: rule.crate_dir
          - rule: crate_dir
            ::
              - dir: queries
                id: queries
                ::
                  - file:
                      regex: '^(?<name>.+)\.sql$'
                    id: q
              - dir: src
                ::
                  - for:
                      id: x
                      value: ${dir.queries.file.q}
                    ::
                      - file: '${value.x.regex.name}_sqlx.rs'
    "#};
    let tree = dir(
        "",
        vec![leaf("queries", &["a.sql"]), leaf("src", &["a_sqlx.rs"])],
        &[],
    );

    // Act
    let errors = run(yaml, &tree);

    // Assert: q is collected on-site even across a splice, and a_sqlx.rs matches with no excess or deficit.
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
