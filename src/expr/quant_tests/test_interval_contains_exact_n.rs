use crate::expr::quant::Interval;

/// min=5, max=Some(5): contains exactly 5 (equivalent to Exact).
#[test]
fn test_interval_contains_exact_n() {
    // Arrange
    let iv = Interval::exactly(5);

    // Act & Assert
    assert!(!iv.contains(4), "4 is below lower bound");
    assert!(iv.contains(5), "5 is exactly the value");
    assert!(!iv.contains(6), "6 exceeds upper bound");
}
