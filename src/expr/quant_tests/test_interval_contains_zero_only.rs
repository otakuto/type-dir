use crate::expr::quant::Interval;

/// min=0, max=Some(0): contains exactly 0 (prohibition interval).
#[test]
fn test_interval_contains_zero_only() {
    // Arrange
    let iv = Interval {
        min: 0,
        max: Some(0),
    };

    // Act & Assert
    assert!(iv.contains(0), "0 is contained");
    assert!(!iv.contains(1), "1 exceeds upper bound");
}
