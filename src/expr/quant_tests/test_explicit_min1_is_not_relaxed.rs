use crate::expr::quant::{Interval, Quant};

/// `Quant::Explicit` with min=1 returns false from `is_relaxed()`.
#[test]
fn test_explicit_min1_is_not_relaxed() {
    // Arrange
    let q = Quant::Explicit(Interval { min: 1, max: None });

    // Act & Assert
    assert!(!q.is_relaxed());
}
