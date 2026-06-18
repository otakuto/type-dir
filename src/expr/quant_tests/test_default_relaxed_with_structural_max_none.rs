use crate::expr::quant::{Interval, Quant};

/// `Quant::Default.relaxed(None)` returns `Explicit { min: 0, max: None }` (optional splice for Regex).
#[test]
fn test_default_relaxed_with_structural_max_none() {
    // Arrange
    let q = Quant::Default;

    // Act
    let relaxed = q.relaxed(None);

    // Assert
    assert_eq!(relaxed, Quant::Explicit(Interval { min: 0, max: None }));
}
