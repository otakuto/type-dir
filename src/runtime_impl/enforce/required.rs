use std::path::Path;

use crate::error::LintError;
use crate::expr::ExprMatcher;
use crate::runtime_impl::name_matcher::{MatchKind, ResolvedMatcher};
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::Value;
use crate::walk::DirTree;

use super::eval::build_effective_chain;
use super::expand::ScopedEntry;
use super::matcher::resolve_entry_matcher;

/// Checks for the presence of required entries (those with no explicit count and an Exact matcher).
///
/// `pairs` is the list of `ScopedEntry` values already expanded by `expand_entries`.
/// Each entry is evaluated against its paired scope; references are extracted from `Cow` via `as_ref()`.
///
/// `node_rule` is the rule name applied to this node. The `rule` field of `MissingRequired` is set to
/// the entry's origin (splice target rule name) if present, otherwise to `node_rule` (used for note resolution).
pub fn check_required_pairs(
    tree: &DirTree,
    pairs: &[ScopedEntry],
    path: &Path,
    node_rule: &str,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
) {
    for (entry, entry_scope, origin, _) in pairs {
        let entry = entry.as_ref();
        let entry_scope = entry_scope.as_ref();
        // Choice entries are handled in the choice section of eval_node, so skip them here.
        if matches!(entry.matcher, ExprMatcher::Choice { .. }) {
            continue;
        }
        // Match entries are expanded by expand_entries into the winning arm's subtree, so they
        // never reach the required check. Skip defensively in case an unexpanded Match appears.
        if matches!(entry.matcher, ExprMatcher::Match { .. }) {
            continue;
        }
        // Entries with an explicit count (Quant::Explicit) are checked via the count constraint (E019),
        // so exclude them from the implicit required check (E002) to avoid duplicate reports.
        // Optional desugared entries (`Explicit { min: 0, max: Some(1) }`) are also excluded here.
        if entry.count.is_explicit() {
            continue;
        }

        // Optional splice propagation: entries that belong to the body of a relax context (an optional
        // `- use:`) have their required check relaxed (no MissingRequired is emitted). The relaxed frame
        // is included in the scope snapshot taken during evaluation of the body directly under the splice.
        if entry_scope.relaxed() {
            continue;
        }

        // Use resolve_entry_matcher to obtain both the expected name and the dir/file distinction.
        // The required check only applies to Exact matchers; Regex matchers are excluded.
        let (name, is_dir) = match resolve_entry_matcher(entry, entry_scope) {
            Some(ResolvedMatcher::Dir(MatchKind::Exact(name))) => (name, true),
            Some(ResolvedMatcher::File(MatchKind::Exact(name))) => (name, false),
            // Regex matchers and entries with no matcher are excluded from the required check.
            _ => continue,
        };

        // If the expected name is empty, do not emit an error.
        if name.is_empty() {
            continue;
        }

        let exists = if is_dir {
            tree.dirs.iter().any(|d| d.name == name)
        } else {
            tree.files.iter().any(|f| f == &name)
        };
        if !exists {
            // Applied rule: use origin (splice target) if present, otherwise use node_rule.
            let rule = origin
                .as_ref()
                .map_or_else(|| node_rule.to_string(), |n| n.0.clone());
            // Build for-binding provenance context from Record bindings in scope.
            // Only Value::Record bindings are included (structural source elements).
            // Sorted by variable name for deterministic output. For-iteration variables live under
            // `(NodeKind::Value, var)`, so their bare name is shown directly.
            let context = {
                let mut parts: Vec<String> = entry_scope
                    .iter_lex()
                    .filter_map(|(kind, var, val)| {
                        if kind != NodeKind::Value {
                            return None;
                        }
                        if let Value::Record(r) = val {
                            if r.whole().is_empty() {
                                None
                            } else {
                                Some(format!("{}={}", var, r.whole()))
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                parts.sort();
                parts.join(", ")
            };
            errors.push(LintError::MissingRequired {
                parent: path.to_path_buf(),
                name,
                is_dir,
                rule,
                context,
                rule_chain: build_effective_chain(rule_chain, origin.as_ref()),
                entry_path: entry.source_path.clone(),
            });
        }
    }
}
