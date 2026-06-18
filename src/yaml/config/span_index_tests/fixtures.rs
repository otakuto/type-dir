/// Sample config YAML shared across span_index tests.
pub(super) const SAMPLE_YAML: &str = r#"version: 0
entry: root
rules:
  - rule: root
    ::
      - dir: src
        ::
          - file: main.rs
      - use: rule.child_rule
  - rule: child_rule
    ::
      - file: foo.rs
"#;
