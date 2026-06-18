use crate::expr::quant::Interval;

/// `Interval::relax_min()` works correctly when max=None.
#[test]
fn test_relax_min_unbounded_stays_unbounded() {
    // Arrange
    let iv = Interval { min: 1, max: None };

    // Act
    let relaxed = iv.relax_min();

    // Assert
    assert_eq!(relaxed.min, 0);
    assert_eq!(relaxed.max, None);
}
