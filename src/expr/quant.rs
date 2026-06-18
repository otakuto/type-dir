#[cfg(test)]
#[path = "quant_tests/tests.rs"]
mod tests;

/// Count interval constraint `[min, max]` (`max: None` means unbounded ∞).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    pub min: usize,
    pub max: Option<usize>,
}

impl Interval {
    /// Returns whether `c` falls within the interval `[min, max]`.
    pub fn contains(&self, c: usize) -> bool {
        c >= self.min && self.max.is_none_or(|m| c <= m)
    }

    /// Returns an interval with the lower bound relaxed to 0 (used for optional splice propagation).
    pub fn relax_min(self) -> Interval {
        Interval {
            min: 0,
            max: self.max,
        }
    }

    /// Returns an interval representing exactly `n`.
    pub fn exactly(n: usize) -> Interval {
        Interval {
            min: n,
            max: Some(n),
        }
    }

    /// Returns an interval representing at least `n`.
    pub fn at_least(n: usize) -> Interval {
        Interval { min: n, max: None }
    }
}

/// Quantifier for an entry.
///
/// `Default` represents user-unspecified; the effective interval is determined by the pattern's
/// structural upper bound and context.
/// `Explicit` holds an explicitly specified interval.
///
/// The legacy `optional` bool flag is desugared at compile time to
/// `Explicit(Interval { min: 0, max: Some(1) })` and does not exist in the AST layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quant {
    /// User-unspecified (effective interval is context-dependent;
    /// absence of Exact is reported as E002, violation of explicit is reported as E019).
    Default,
    /// Explicitly specified interval.
    Explicit(Interval),
}

impl Quant {
    /// Resolves to the effective interval given the structural upper bound.
    ///
    /// - `Default` → `{ min: 1, max: structural_max }`
    /// - `Explicit(iv)` → `iv` (structural upper bound is ignored)
    #[allow(dead_code)]
    pub fn effective(&self, structural_max: Option<usize>) -> Interval {
        match self {
            Quant::Default => Interval {
                min: 1,
                max: structural_max,
            },
            Quant::Explicit(iv) => *iv,
        }
    }

    /// Returns whether the quantifier is explicitly specified.
    pub fn is_explicit(&self) -> bool {
        matches!(self, Quant::Explicit(_))
    }

    /// Returns whether the effective lower bound is 0 (equivalent to optional).
    ///
    /// Returns `true` only when `Explicit` has `min == 0`. Always `false` for `Default`.
    pub fn is_relaxed(&self) -> bool {
        matches!(self, Quant::Explicit(iv) if iv.min == 0)
    }

    /// Returns a `Quant` with the lower bound relaxed to 0.
    ///
    /// - `Default` is first resolved to `Explicit` using the structural upper bound, then min is lowered to 0.
    /// - `Explicit(iv)` returns `iv.relax_min()`.
    #[allow(dead_code)]
    pub fn relaxed(self, structural_max: Option<usize>) -> Quant {
        let iv = self.effective(structural_max);
        Quant::Explicit(iv.relax_min())
    }
}
