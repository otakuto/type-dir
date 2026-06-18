use crate::expr::quant::Quant;

/// `Quant::Default` returns false from `is_relaxed()`.
#[test]
fn test_default_is_not_relaxed() {
    // Arrange
    let q = Quant::Default;

    // Act & Assert
    assert!(!q.is_relaxed());
}
