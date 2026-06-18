use super::super::strip_last_segment;

#[test]
fn strip_last_segment_root_returns_none() {
    // Arrange / Act / Assert
    assert_eq!(strip_last_segment("rules"), None);
}
