use crate::expr::quant::Interval;

/// `Interval::relax_min()` lowers min to 0 without changing max.
#[test]
fn test_relax_min_zeroes_lower_bound() {
    // Arrange
    let iv = Interval {
        min: 3,
        max: Some(5),
    };

    // Act
    let relaxed = iv.relax_min();

    // Assert
    assert_eq!(relaxed.min, 0);
    assert_eq!(relaxed.max, Some(5));
}
