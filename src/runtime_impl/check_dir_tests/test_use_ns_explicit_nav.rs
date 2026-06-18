use super::{dir, leaf, run};

/// Verifies that `${use.<id>}` references the splice instance wrapper and the desired record set is
/// reached by explicit navigation (`.dir.<id>`), with no magic auto-unwrap.
///
/// A splice (`- use: rule.X / id: Y`) binds the wrapper record `Y` as a self-owned id. The wrapper's
/// public id (e.g. `node`) is reached via `${use.Y.dir.node}` — the same record set that the
/// `${rule.X.dir.node}` type path describes. Auto-unwrap (bare `${use.Y}`) is no longer supported;
/// the navigation must be spelled out.
///
/// This test uses a simple rule `rec` that captures dir names as records under `id: node`.
/// The `root` rule splices `rec` with `id: rec`, making `rec` a self-owned id. The `out` dir
/// then uses `for n in ${use.rec.dir.node}` to iterate the captured node records.
///
/// Directory layout:
/// ```text
/// <root>/
///   src/          ← matched by rule rec (node records captured here)
///     a/          ← node record with stem="a"
///     b/          ← node record with stem="b"
///   out/
///     a-dir/      ← expected via for n in ${use.rec.dir.node} :: dir '${n.regex.stem}-dir'
///     b-dir/
/// ```
const USE_NS_EXPLICIT_NAV_YAML: &str = indoc::indoc! {r#"
    version: 0
    entry: root
    rules:
      - rule: root
        ::
          - use: rule.rec
            id: rec
          - dir: out
            ::
              - for:
                  id: n
                  value: ${use.rec.dir.src_inner.dir.node}
                ::
                  - dir: '${value.n.regex.stem}-dir'

      - rule: rec
        ::
          - dir: src
            id: src_inner
            ::
              - dir:
                  regex: '^(?<stem>[a-z]+)$'
                id: node
                optional: true
"#};

/// `${use.rec.dir.node}` navigates the wrapper to the node record set and the for iterates them.
#[test]
fn use_ns_explicit_nav_iterates_records() {
    // Arrange: src/a and src/b → node records with stem="a" and stem="b".
    // out/ must contain a-dir and b-dir.
    let src = dir("src", vec![leaf("a", &[]), leaf("b", &[])], &[]);
    let out = dir("out", vec![leaf("a-dir", &[]), leaf("b-dir", &[])], &[]);
    let tree = dir("", vec![src, out], &[]);

    // Act
    let errors = run(USE_NS_EXPLICIT_NAV_YAML, &tree);

    // Assert: no diagnostics — explicit nav resolves to the node record set and for iterates them.
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

/// When the splice instance has no matching node records, `${use.rec.dir.node}` is the empty set and
/// `for n in ${use.rec.dir.node}` iterates zero times, leaking no unresolved `${n.regex.stem}-dir`.
#[test]
fn use_ns_explicit_nav_empty_set_zero_iterations() {
    // Arrange: src/ is empty → node (optional) matches zero times → the wrapper has no `node` child
    // records → ${use.rec.dir.node} is the empty set → for runs zero times → out/ stays empty.
    let src = leaf("src", &[]);
    let out = leaf("out", &[]);
    let tree = dir("", vec![src, out], &[]);

    // Act
    let errors = run(USE_NS_EXPLICIT_NAV_YAML, &tree);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
