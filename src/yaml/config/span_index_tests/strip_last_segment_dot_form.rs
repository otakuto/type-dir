use super::super::strip_last_segment;

#[test]
fn strip_last_segment_dot_form() {
    // Arrange / Act / Assert
    assert_eq!(strip_last_segment("rules.foo.rules"), Some("rules.foo"));
}
