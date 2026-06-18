#[cfg(test)]
#[path = "name_matcher_tests/tests.rs"]
mod tests;

use std::collections::HashMap;

use crate::expr::ExprPattern;
use crate::runtime_impl::regex_cache::regex_named_captures;

use super::template::{substitute, substitute_regex_literal};

/// Match variant holding strings after `${var}` substitution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchKind {
    /// Exact-match matcher.
    Exact(String),
    /// Regex matcher.
    Regex(String),
}

/// The effective name matcher of an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedMatcher {
    /// Matcher applied to a directory name.
    Dir(MatchKind),
    /// Matcher applied to a file name.
    File(MatchKind),
}

/// Converts an `ExprPattern` to a `MatchKind`, applying `${var}` substitution from the scope.
pub fn resolve_pattern(pattern: &ExprPattern, scope: &HashMap<String, String>) -> MatchKind {
    match pattern {
        ExprPattern::Exact(s) => MatchKind::Exact(substitute(s, scope)),
        // In regex context, `${var}` values are literal-escaped (regex::escape) before embedding.
        ExprPattern::Regex(regex) => MatchKind::Regex(substitute_regex_literal(&regex.0, scope)),
    }
}

/// Common function for matching a directory or file name against a `MatchKind`.
///
/// Returns a captures map on match, or `None` if no match.
/// For `Regex`, the map contains group `"0"` (full match), positional groups, and named groups.
/// For `Exact`, the map contains `"0"` set to the matched name (the full match equals the name itself).
pub fn match_match_kind(kind: &MatchKind, name: &str) -> Option<HashMap<String, String>> {
    match kind {
        MatchKind::Exact(expected) => {
            if name == expected.as_str() {
                Some(HashMap::from([("0".to_string(), name.to_string())]))
            } else {
                None
            }
        }
        MatchKind::Regex(pattern) => regex_named_captures(pattern, name),
    }
}
