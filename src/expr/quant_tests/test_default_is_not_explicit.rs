use crate::expr::quant::Quant;

/// `Quant::Default` returns false from `is_explicit()`.
#[test]
fn test_default_is_not_explicit() {
    // Arrange
    let q = Quant::Default;

    // Act & Assert
    assert!(!q.is_explicit());
}
