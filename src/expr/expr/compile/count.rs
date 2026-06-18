use crate::expr::{Interval, Quant};
use crate::yaml::{YamlEntry, YamlEntryKind, YamlPattern};

/// Normalizes the four entry keys (`optional` / `min` / `max` / `count`) into `Quant`.
///
/// When all keys are absent (user-unspecified), returns `Quant::Default`. At runtime,
/// `Quant::Default` applies the default rule (Exact=required, regex={1,∞}).
pub fn normalize_count(entry: &YamlEntry) -> Quant {
    // If count scalar is specified, fix to {n,n} (XOR is pre-guaranteed).
    if let Some(n) = entry.count {
        return Quant::Explicit(Interval::exactly(n));
    }

    let has_optional = entry.optional.is_some();
    let has_min = entry.min.is_some();
    let has_max = entry.max.is_some();

    // All keys absent → Default (user-unspecified)
    if !has_optional && !has_min && !has_max {
        return Quant::Default;
    }

    // optional: false is explicit but equivalent to min=1 (not optional)
    let is_optional = entry.optional == Some(true);
    let min_eff = if has_min {
        entry.min.expect("has_min guard guarantees min is Some")
    } else if is_optional {
        0
    } else {
        1
    };

    // Use explicit max, or derive from structure (Exact=1, others=∞=None)
    let is_exact = matches!(&entry.kind, YamlEntryKind::Dir { pattern, .. } | YamlEntryKind::File { pattern, .. } if YamlPattern::is_exact(pattern));
    let max_eff = if has_max {
        entry.max
    } else if is_exact {
        Some(1)
    } else {
        None
    };

    Quant::Explicit(Interval {
        min: min_eff,
        max: max_eff,
    })
}
