use crate::expr::quant::{Interval, Quant};

/// `Quant::Explicit({ min: 0, max: Some(1) }).relaxed(_)` is idempotent since min is already 0.
#[test]
fn test_explicit_already_relaxed_is_idempotent() {
    // Arrange
    let q = Quant::Explicit(Interval {
        min: 0,
        max: Some(1),
    });

    // Act
    let relaxed = q.relaxed(None);

    // Assert
    assert_eq!(
        relaxed,
        Quant::Explicit(Interval {
            min: 0,
            max: Some(1)
        })
    );
}
