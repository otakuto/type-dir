use crate::expr::quant::Interval;

/// min=1, max=Some(3): covers boundary values and out-of-range cases.
#[test]
fn test_interval_contains_within_range() {
    // Arrange
    let iv = Interval {
        min: 1,
        max: Some(3),
    };

    // Act & Assert
    assert!(!iv.contains(0), "0 is below lower bound");
    assert!(iv.contains(1), "1 is exactly the lower bound");
    assert!(iv.contains(2), "2 is within range");
    assert!(iv.contains(3), "3 is exactly the upper bound");
    assert!(!iv.contains(4), "4 exceeds upper bound");
}
