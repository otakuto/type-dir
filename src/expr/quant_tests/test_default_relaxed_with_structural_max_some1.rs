use crate::expr::quant::{Interval, Quant};

/// `Quant::Default.relaxed(Some(1))` returns `Explicit { min: 0, max: Some(1) }`.
#[test]
fn test_default_relaxed_with_structural_max_some1() {
    // Arrange
    let q = Quant::Default;

    // Act
    let relaxed = q.relaxed(Some(1));

    // Assert
    assert_eq!(
        relaxed,
        Quant::Explicit(Interval {
            min: 0,
            max: Some(1)
        })
    );
}
