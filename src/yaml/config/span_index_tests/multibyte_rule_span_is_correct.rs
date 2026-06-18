use super::super::build_span_index;

// YAML source that has multibyte (Japanese) content before the rule definition.
// The comments contain multibyte characters to ensure char offsets diverge from byte offsets.
const MULTIBYTE_YAML: &str = "version: 0\nentry: root\n# あいうえおかきくけこ あいうえおかきくけこ\n# さしすせそたちつてと さしすせそたちつてと\nrules:\n  - rule: target\n    ::\n      - file: foo.rs\n";

#[test]
fn multibyte_rule_span_is_correct() {
    let index = build_span_index(MULTIBYTE_YAML);

    let span = index
        .lookup_with_ancestors("rules.target")
        .expect("rules.target must be indexed");

    // The stored offsets must be valid byte boundaries in the source string.
    assert!(
        MULTIBYTE_YAML.is_char_boundary(span.start),
        "span.start ({}) is not a char boundary",
        span.start,
    );
    assert!(
        MULTIBYTE_YAML.is_char_boundary(span.end),
        "span.end ({}) is not a char boundary",
        span.end,
    );

    // The slice at those byte offsets must contain the rule definition line.
    let slice = &MULTIBYTE_YAML[span.start..span.end];
    assert!(
        slice.contains("- rule: target"),
        "expected slice to contain '- rule: target', got: {:?}",
        slice,
    );
}
