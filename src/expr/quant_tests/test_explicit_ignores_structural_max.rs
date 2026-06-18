use crate::expr::quant::{Interval, Quant};

/// `Quant::Explicit` ignores structural_max and returns its own interval.
#[test]
fn test_explicit_ignores_structural_max() {
    // Arrange
    let iv_in = Interval {
        min: 2,
        max: Some(4),
    };
    let q = Quant::Explicit(iv_in);

    // Act
    let iv = q.effective(Some(1));

    // Assert: structural_max=Some(1) is ignored
    assert_eq!(iv.min, 2);
    assert_eq!(iv.max, Some(4));
}
