use crate::expr::quant::Interval;

/// `Interval::exactly(n)` returns `[n, n]`.
#[test]
fn test_exactly_produces_symmetric_interval() {
    // Arrange
    let n = 3;

    // Act
    let iv = Interval::exactly(n);

    // Assert
    assert_eq!(iv.min, 3);
    assert_eq!(iv.max, Some(3));
}
