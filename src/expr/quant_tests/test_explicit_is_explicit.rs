use crate::expr::quant::{Interval, Quant};

/// `Quant::Explicit` returns true from `is_explicit()`.
#[test]
fn test_explicit_is_explicit() {
    // Arrange
    let q = Quant::Explicit(Interval::exactly(1));

    // Act & Assert
    assert!(q.is_explicit());
}
