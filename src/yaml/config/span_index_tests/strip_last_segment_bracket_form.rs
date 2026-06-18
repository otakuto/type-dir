use super::super::strip_last_segment;

#[test]
fn strip_last_segment_bracket_form() {
    // Arrange / Act / Assert
    assert_eq!(
        strip_last_segment("rules.foo.rules[2]"),
        Some("rules.foo.rules")
    );
}
