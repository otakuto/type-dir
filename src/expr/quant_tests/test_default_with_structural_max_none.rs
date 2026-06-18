use crate::expr::quant::Quant;

/// `Quant::Default` returns `[1, ∞)` when structural_max=None (Regex).
#[test]
fn test_default_with_structural_max_none() {
    // Arrange
    let q = Quant::Default;

    // Act
    let iv = q.effective(None);

    // Assert
    assert_eq!(iv.min, 1);
    assert_eq!(iv.max, None);
}
