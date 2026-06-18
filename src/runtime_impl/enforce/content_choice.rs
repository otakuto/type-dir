use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprRule};
use crate::runtime_impl::env::Scope;
use crate::walk::DirTree;
use crate::yaml::RuleName;

use super::eval::eval_node_chained;
use super::memo::{TrialMemo, fingerprint_scope, memo_node_id};
use super::with::build_with_scope;

/// Evaluates content-choice mode.
///
/// Trial-validates the contents of the current node `tree` against the rule of each alternative (`Splice`)
/// and counts the number of valid (zero-error) alternatives. `one_of` (min=1,max=1) requires exactly 1 valid;
/// `any_of` (min=1,max=None) requires at least 1 valid.
///
/// - If the valid count is below the lower bound, reports the error from the closest (fewest errors)
///   branch.
/// - If the valid count exceeds max (multiple matches in one_of = ambiguous), reports `CardinalityViolation`.
///
/// Validation of each alternative is memoized per `(node, rule, σ)` (`TrialMemo`) to prevent exponential
/// re-execution of the same trial in nested content-choice. `dirs` traces inside trials are discarded and
/// not included in results; only errors are cached.
#[allow(clippy::too_many_arguments)]
pub fn eval_content_choice(
    tree: &DirTree,
    alternatives: &[ExprEntry],
    min: usize,
    max: Option<usize>,
    scope: &Scope,
    rules: &IndexMap<RuleName, ExprRule>,
    path: &Path,
    _rule_name: &str,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
    memo: &mut TrialMemo,
) {
    let mut valid = 0usize;
    // Retain the closest (fewest errors) failed trial.
    let mut closest: Option<Vec<LintError>> = None;

    for alt in alternatives {
        let ExprMatcher::Use { rule, with_args } = &alt.matcher else {
            continue;
        };
        let Some(rule_def) = rules.get(rule) else {
            continue;
        };
        let s_scope = build_with_scope(rule_def, with_args, scope);

        // Build the memo key as `(node identity, rule name, σ fingerprint)`.
        let key = (
            memo_node_id(tree),
            rule.clone(),
            fingerprint_scope(&s_scope),
        );
        let trial = if let Some(cached) = memo.get(&key) {
            cached.clone()
        } else {
            let mut fresh = Vec::new();
            let mut trial_produced = crate::runtime_impl::record_map::RecordMap::new();
            // The content-choice trial expands rule N's body at this node, so the child chain
            // is built by appending N to the current chain (avoiding tail duplicates).
            let mut child_chain = rule_chain.to_vec();
            if child_chain.last().map(|s| s.as_str()) != Some(rule.0.as_str()) {
                child_chain.push(rule.0.clone());
            }
            eval_node_chained(
                tree,
                &rule_def.rules,
                &s_scope,
                rules,
                path,
                &rule.0,
                &child_chain,
                &mut fresh,
                &mut trial_produced,
                memo,
            );
            memo.insert(key, fresh.clone());
            fresh
        };

        if trial.is_empty() {
            valid += 1;
        } else if closest.as_ref().is_none_or(|c| trial.len() < c.len()) {
            closest = Some(trial);
        }
    }

    let below = valid < min;
    let above = max.is_some_and(|m| valid > m);

    if below {
        // When below the lower bound, report the closest (fewest errors) failed branch's diagnostics.
        if let Some(errs) = closest {
            errors.extend(errs);
        }
    } else if above {
        // Multiple matches in one_of (ambiguous): report as cardinality constraint violation.
        errors.push(LintError::CardinalityViolation {
            parent: path.to_path_buf(),
            realized: valid,
            min,
            max,
            rule_chain: rule_chain.to_vec(),
            entry_path: None,
        });
    }
}
