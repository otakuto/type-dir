use crate::expr::ExprEntry;
use crate::yaml::YamlEntry;

use super::count::normalize_count;
use super::matcher::build_matcher;

/// Converts a YAML entry to an ExprEntry (assumes validation has passed).
///
/// Normalizes the four keys (`optional` / `min` / `max` / `count`) into `ExprEntry.count: Quant`.
/// Normalization rules:
/// - All keys absent → `Quant::Default` (runtime applies default min=1 or structural max)
/// - `count: n` → `Quant::Explicit(Interval::exactly(n))`
/// - Other combinations of (`optional` / `min` / `max`) →
///   min_eff = explicit min, or 0 if optional:true, or 1
///   max_eff = explicit max, or structural default (Exact→Some(1), regex→None)
///
/// XOR constraints are pre-guaranteed by check_entry_combination, so mutual exclusion is assumed here.
///
/// For record-intro entries (anonymous group or splice+id desugared to Record), `ExprEntry.id`
/// carries the record id from the YAML entry. Bare Splice entries (no id) have `ExprEntry.id = None`.
pub fn to_expr_entry(entry: &YamlEntry, path: Option<&str>) -> ExprEntry {
    let matcher = build_matcher(entry, path);
    let count = normalize_count(entry);
    // Bare Use entries (no dir/file, bare rule, no id) produce ExprMatcher::Use and have no id.
    // Use+id entries produce ExprMatcher::Group (the id wraps the use entry), so `entry.id` is used.
    // Fetch entries now carry their id in entry.id (moved from old fetch tuple first element).
    // All other matchers (Own, Group, Choice, For, Match) carry entry.id directly.
    let id = match &matcher {
        crate::expr::ExprMatcher::Use { .. } => None,
        _ => entry.id.clone(),
    };
    ExprEntry {
        id,
        source_path: path.map(|s| s.to_string()),
        count,
        matcher,
    }
}
