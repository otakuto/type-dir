use super::{dir, leaf, run};

/// Regression test for the Γ_set asymmetry in `resolve_expr_source`.
///
/// An `id:`-bearing `dir` entry stores its records in Γ_set.  A sibling `for` then iterates via a
/// **qualified hop** `${q.file.qf}` — i.e. it starts from the splice id set `q` (held in Γ_set)
/// and descends through `.file.qf`.  Before the fix, `resolve_expr_source` only called
/// `scope.lookup_lex("q")` which returns `None` for Γ_set entries, so the loop silently produced
/// zero iterations.  After the fix, `head_records` is used instead, which also consults Γ_set.
///
/// `qf` is a `file:` capture, so the reference uses `.file.qf` to keep the static kind check
/// (`NodeKindMismatch`) satisfied.
///
/// Directory layout:
/// ```text
/// <root>/
///   queries/          ← matched by id:q (its record is stored in Γ_set["q"])
///     a.sql           ← matched by file id:qf inside queries (child record of q)
///   src/
///     a_sqlx.rs       ← expected via `for x in ${q.file.qf} :: file '${x.regex.name}_sqlx.rs'`
/// ```
const SPLICE_ID_QUALIFIED_HOP_YAML: &str = indoc::indoc! {r#"
    version: 0
    entry: root
    rules:
      - rule: root
        ::
          - dir: queries
            id: q
            ::
              - file:
                  regex: '^(?<name>.+)\.sql$'
                id: qf
          - dir: src
            ::
              - for:
                  id: x
                  value: ${dir.q.file.qf}
                ::
                  - file: '${value.x.regex.name}_sqlx.rs'
"#};

/// Verifies that `${q.file.qf}` (qualified hop from Γ_set entry `q`) iterates child records and
/// produces the correct `Own` entries.  Before the bug fix this test emitted a spurious
/// `UndeclaredEntry` for `a_sqlx.rs` and a `MissingRequired` because the loop ran zero times.
#[test]
fn for_source_splice_id_qualified_hop_iterates_records() {
    // Arrange: `queries/a.sql` populates Γ_set["q"] with a record whose child set "qf" contains
    // the record for `a.sql` (fields["name"] = "a").  `src/a_sqlx.rs` is the expected output file.
    let queries = leaf("queries", &["a.sql"]);
    let src = leaf("src", &["a_sqlx.rs"]);
    let tree = dir("", vec![queries, src], &[]);

    // Act
    let errors = run(SPLICE_ID_QUALIFIED_HOP_YAML, &tree);

    // Assert: no diagnostics — the qualified-hop for-source found Γ_set records and matched the file.
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

/// Same shape as `SPLICE_ID_QUALIFIED_HOP_YAML`, but `qf` is `optional: true` (effective min 0).
///
/// This isolates the empty-set behaviour of the for-source: with the default min 1, an empty
/// `queries` dir would emit an unrelated `CountViolation { parent: "queries", min: 1 }`, masking
/// the actual property under test (that `for x in ${q.file.qf}` runs zero times and emits no
/// false diagnostics).  Making `qf` optional removes that interference.
const SPLICE_ID_QUALIFIED_HOP_OPTIONAL_YAML: &str = indoc::indoc! {r#"
    version: 0
    entry: root
    rules:
      - rule: root
        ::
          - dir: queries
            id: q
            ::
              - file:
                  regex: '^(?<name>.+)\.sql$'
                id: qf
                optional: true
          - dir: src
            ::
              - for:
                  id: x
                  value: ${dir.q.file.qf}
                ::
                  - file: '${value.x.regex.name}_sqlx.rs'
"#};

/// When `queries` is empty, `q`'s child set `qf` is empty, so the for loop runs zero times.
/// The empty `src` directory must be accepted without diagnostics. `qf` is `optional: true` so the
/// empty `queries` dir does not produce an unrelated `CountViolation`.
#[test]
fn for_source_splice_id_qualified_hop_empty_set_zero_iterations() {
    // Arrange: no .sql files → Γ_set["q"] exists but its child set "qf" is empty.
    let queries = leaf("queries", &[]);
    let src = leaf("src", &[]);
    let tree = dir("", vec![queries, src], &[]);

    // Act
    let errors = run(SPLICE_ID_QUALIFIED_HOP_OPTIONAL_YAML, &tree);

    // Assert: zero iterations means the for body never expands → no required entries, and the
    // optional `qf` emits no CountViolation → errors is completely empty.
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
