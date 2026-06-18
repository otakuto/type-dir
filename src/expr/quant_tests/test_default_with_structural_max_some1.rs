use crate::expr::quant::Quant;

/// `Quant::Default` returns `[1, 1]` when structural_max=Some(1) (Exact structural upper bound).
#[test]
fn test_default_with_structural_max_some1() {
    // Arrange
    let q = Quant::Default;

    // Act
    let iv = q.effective(Some(1));

    // Assert
    assert_eq!(iv.min, 1);
    assert_eq!(iv.max, Some(1));
}
