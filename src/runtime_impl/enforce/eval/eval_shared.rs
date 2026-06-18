use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, Quant};
use crate::yaml::RuleName;

use super::super::assignment_counts::AssignmentCounts;
use super::super::expand::ScopedEntry;
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::Record;
use crate::runtime_impl::visible_ids::collect_visible_ids;

/// Returns the first tag found in a body RecordMap.
///
/// Used when determining the tag of a for-record that contains the body of an id-bearing Group
/// (one_of / any_of). Returns the tag if any Record in `child_out` has one set, otherwise returns `None`.
pub(super) fn lift_tag_from_child_out(
    child_out: &crate::runtime_impl::record_map::RecordMap,
) -> Option<String> {
    for records in child_out.values() {
        for rec in records {
            if rec.tag.is_some() {
                return rec.tag.clone();
            }
        }
    }
    None
}

/// Returns `N` if the entry list is non-empty and every entry is a bare use to the same rule `N`.
///
/// Used in step (a) of child_rule determination when descending into a child dir.
/// Returns `Some(N)` when the list consists only of `- use: rule.N` Use entries (count presence is ignored)
/// and all point to the same `N`. Returns `None` for inline entries, dir/file/group/for mixtures,
/// empty lists, or multiple rules.
pub(super) fn dominant_use_rule(entries: &[ExprEntry]) -> Option<&crate::yaml::RuleName> {
    let mut iter = entries.iter().map(|e| match &e.matcher {
        ExprMatcher::Use { rule, .. } => Some(rule),
        ExprMatcher::Match { .. } => None,
        _ => None,
    });
    let first = iter.next()??;
    if iter.all(|r| r == Some(first)) {
        Some(first)
    } else {
        None
    }
}

pub(super) fn into_static_scoped(scoped: ScopedEntry<'_>) -> ScopedEntry<'static> {
    let (entry, scope, origin, owner) = scoped;
    (
        Cow::Owned(entry.into_owned()),
        Cow::Owned(scope.into_owned()),
        origin,
        owner,
    )
}

/// Builds an `id → NodeKind` lookup table for the visible ids of an entry list.
///
/// Because produced (RecordMap) carries only the id without a kind, this table is consulted when
/// reflecting produced into work_scope to look up each id's producer kind. Ids absent from the
/// table are defensively treated as `NodeKind::Regex` (rare cases that appear across encapsulation
/// boundaries).
pub(super) fn visible_kind_map(entries: &[ExprEntry]) -> HashMap<String, NodeKind> {
    collect_visible_ids(entries)
        .into_iter()
        .map(|(kind, id)| (id, kind))
        .collect()
}

/// Cumulatively reflects produced (RecordMap) into work_scope according to the visible kind table.
pub(super) fn bind_produced_into_scope(
    work_scope: &mut Scope,
    kind_of: &HashMap<String, NodeKind>,
    id: &str,
    records: &[Record],
) {
    let kind = kind_of.get(id).copied().unwrap_or(NodeKind::Regex);
    work_scope.bind_env(kind, id.to_string(), records.to_vec());
}

/// Checks the interval constraint term c_e∈[min,max].
///
/// Checks the count constraint (E019) for standalone entries (outside groups).
/// Exact + Quant::Default is delegated to the E002 path in required.rs.
pub(super) fn check_interval_constraints(
    expanded: &[ScopedEntry<'_>],
    counts: &AssignmentCounts,
    path: &Path,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
) {
    for (idx, (entry, entry_scope, origin, _)) in expanded.iter().enumerate() {
        let entry = entry.as_ref();
        // group entries are processed in the choice realization section. Only dir/file is targeted here.
        let (pattern,) = match &entry.matcher {
            ExprMatcher::Dir { pattern, .. } | ExprMatcher::File { pattern, .. } => (pattern,),
            _ => continue,
        };
        let mut interval = match entry.count {
            Quant::Explicit(iv) => iv,
            Quant::Default => {
                // Default: Exact is delegated to required.rs. Non-Exact defaults to {1,∞}.
                if matches!(pattern, ExprPattern::Exact(_)) {
                    continue;
                }
                crate::expr::Interval::at_least(1)
            }
        };
        // Optional splice propagation: entries that belong to the body of a relax context (an optional
        // `- use:`) treat min as 0 (relax the lower bound of the interval).
        if entry_scope.as_ref().relaxed() {
            interval = interval.relax_min();
        }
        let observed = counts.entry_count(idx);
        if !interval.contains(observed) {
            errors.push(LintError::CountViolation {
                parent: path.to_path_buf(),
                name: count_entry_name(entry, entry_scope.as_ref()),
                observed,
                min: interval.min,
                max: interval.max,
                rule_chain: build_effective_chain(rule_chain, origin.as_ref()),
                entry_path: entry.source_path.clone(),
            });
        }
    }
}

/// Checks the choice realization term Σ[c_a∈Q_a]∈[min,max].
///
/// For each group (one_of/any_of/choice), counts the number of realized alternatives and
/// checks whether it falls within [min, max] (reports E013 CardinalityViolation).
pub(super) fn check_choice_realization(
    expanded: &[ScopedEntry<'_>],
    counts: &AssignmentCounts,
    path: &Path,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
) {
    for (idx, (entry, _, origin, _)) in expanded.iter().enumerate() {
        let ExprMatcher::Choice { min, max, body, .. } = &entry.as_ref().matcher else {
            continue;
        };
        let realized = body
            .iter()
            .enumerate()
            .filter(|(alt_idx, alt)| {
                let c_a = counts.alt_count(idx, *alt_idx);
                // If the alternative has an Explicit count, use interval check; Default means "at least 1" for realization.
                match alt.count {
                    Quant::Explicit(iv) => iv.contains(c_a),
                    Quant::Default => c_a >= 1,
                }
            })
            .count();
        let below = realized < *min;
        let above = max.is_some_and(|m| realized > m);
        if below || above {
            errors.push(LintError::CardinalityViolation {
                parent: path.to_path_buf(),
                realized,
                min: *min,
                max: *max,
                rule_chain: build_effective_chain(rule_chain, origin.as_ref()),
                entry_path: entry.as_ref().source_path.clone(),
            });
        }
    }
}

/// Builds the effective rule chain for error reporting.
///
/// `rule_chain` represents the directory traversal chain from the outside. `origin` represents the
/// entry's splice origin (the innermost splice target rule name). The result concatenates both,
/// skipping the addition if it would duplicate the tail.
pub(crate) fn build_effective_chain(
    rule_chain: &[String],
    origin: Option<&RuleName>,
) -> Vec<String> {
    let mut result = rule_chain.to_vec();
    if let Some(r) = origin
        && result.last().map(|s| s.as_str()) != Some(r.0.as_str())
    {
        result.push(r.0.clone());
    }
    result
}

/// Appends `next` to the end of the current chain (avoiding tail duplicates).
pub(super) fn append_rule_chain(base: &[String], next: &str) -> Vec<String> {
    let mut ch = base.to_vec();
    if ch.last().map(|s| s.as_str()) != Some(next) {
        ch.push(next.to_string());
    }
    ch
}

fn count_entry_name(entry: &ExprEntry, scope: &Scope) -> String {
    use super::super::matcher::resolve_entry_matcher;
    use crate::runtime_impl::name_matcher::{MatchKind, ResolvedMatcher};

    let kind = match resolve_entry_matcher(entry, scope) {
        Some(ResolvedMatcher::Dir(kind)) | Some(ResolvedMatcher::File(kind)) => kind,
        None => return "<entry>".to_string(),
    };
    match kind {
        MatchKind::Exact(name) => name,
        MatchKind::Regex(pattern) => format!("/{pattern}/"),
    }
}
