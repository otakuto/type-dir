use crate::expr::quant::{Interval, Quant};

/// `Quant::Explicit` with min=0 is detectable via `is_relaxed()`.
#[test]
fn test_explicit_min0_is_relaxed() {
    // Arrange
    let q = Quant::Explicit(Interval {
        min: 0,
        max: Some(1),
    });

    // Act & Assert
    assert!(q.is_relaxed());
}
