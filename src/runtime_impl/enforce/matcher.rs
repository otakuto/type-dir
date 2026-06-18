use std::collections::HashMap;

use crate::expr::{ExprEntry, ExprMatcher, ExprPattern};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::name_matcher::{
    MatchKind, ResolvedMatcher, match_match_kind, resolve_pattern,
};
use crate::runtime_impl::template::resolve_template;

use super::with::scope_to_scalar;

/// Matches a directory name or file name against a `MatchKind`.
/// Returns named captures (for Regex) when the match succeeds.
pub fn match_name(kind: &MatchKind, name: &str) -> Option<HashMap<String, String>> {
    match_match_kind(kind, name)
}

/// Helper that resolves an `ExprPattern` into a `ResolvedMatcher` using a `Scope`.
/// `is_dir` selects whether to fold into `Dir` or `File`.
pub fn resolve_expr_pattern_to_matcher(
    pattern: &ExprPattern,
    is_dir: bool,
    scope: &Scope,
) -> ResolvedMatcher {
    let kind = match pattern {
        // Resolve the template to a scalar and fold into an Exact MatchKind.
        ExprPattern::Exact(tmpl) => MatchKind::Exact(resolve_template(tmpl, scope)),
        // Regex: convert scope to scalar and delegate to the collect-side resolve_pattern (matching is unified into MatchKind).
        ExprPattern::Regex(_) => {
            let scalar_scope = scope_to_scalar(scope);
            resolve_pattern(pattern, &scalar_scope)
        }
    };
    if is_dir {
        ResolvedMatcher::Dir(kind)
    } else {
        ResolvedMatcher::File(kind)
    }
}

/// Resolves the name matcher of an entry and returns a `ResolvedMatcher`.
/// Returns `None` for entries that have no name matcher (group/splice/for).
pub fn resolve_entry_matcher(entry: &ExprEntry, scope: &Scope) -> Option<ResolvedMatcher> {
    match &entry.matcher {
        ExprMatcher::Dir { pattern, .. } => {
            Some(resolve_expr_pattern_to_matcher(pattern, true, scope))
        }
        ExprMatcher::File { pattern, .. } => {
            Some(resolve_expr_pattern_to_matcher(pattern, false, scope))
        }
        // A choice entry itself has no name matcher (alternatives are expanded on the consumer side).
        ExprMatcher::Choice { .. } => None,
        // Use has no name matcher (already expanded by expand_entries / handled by content-choice).
        ExprMatcher::Use { .. } => None,
        // For entries are already expanded by expand_entries and are not reached here.
        ExprMatcher::For { .. } => None,
        // Group entries are already expanded by expand_entries and are not reached here.
        ExprMatcher::Group { .. } => None,
        // Match entries are already expanded by expand_entries (into the winning arm's subtree)
        // and are not reached here.
        ExprMatcher::Match { .. } => None,
        // Fetch entries are observation-only and have no name matcher.
        ExprMatcher::Fetch { .. } => None,
        // Value entries are consumed by expand_entries (non-consuming value binding) and are not
        // reached here; defensively report no name matcher (same as Record/Match).
        ExprMatcher::Value { .. } => None,
    }
}

/// Determines whether `entry` matches `child_name` (dir or file).
pub fn entry_matches_name(
    entry: &ExprEntry,
    child_name: &str,
    scope: &Scope,
    is_dir: bool,
) -> Option<HashMap<String, String>> {
    match resolve_entry_matcher(entry, scope) {
        Some(ResolvedMatcher::Dir(kind)) if is_dir => match_name(&kind, child_name),
        Some(ResolvedMatcher::File(kind)) if !is_dir => match_name(&kind, child_name),
        _ => None,
    }
}
