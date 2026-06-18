use crate::expr::quant::Interval;

/// min=0, max=None: unbounded upper bound contains all non-negative integers.
#[test]
fn test_interval_contains_unbounded_max() {
    // Arrange
    let iv = Interval { min: 0, max: None };

    // Act & Assert
    assert!(iv.contains(0), "0 is contained");
    assert!(iv.contains(1000), "any large value is also contained");
}
