use crate::expr::quant::Interval;

/// `Interval::at_least(n)` returns `[n, ∞)`.
#[test]
fn test_at_least_produces_unbounded_interval() {
    // Arrange
    let n = 2;

    // Act
    let iv = Interval::at_least(n);

    // Assert
    assert_eq!(iv.min, 2);
    assert_eq!(iv.max, None);
}
