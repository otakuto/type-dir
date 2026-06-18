use crate::expr::quant::{Interval, Quant};

/// `Quant::Explicit({ min: 2, max: Some(4) }).relaxed(_)` lowers min to 0 and keeps max.
#[test]
fn test_explicit_relaxed_zeroes_min_keeps_max() {
    // Arrange
    let q = Quant::Explicit(Interval {
        min: 2,
        max: Some(4),
    });

    // Act
    let relaxed = q.relaxed(Some(1)); // structural_max is ignored

    // Assert
    assert_eq!(
        relaxed,
        Quant::Explicit(Interval {
            min: 0,
            max: Some(4)
        })
    );
}
