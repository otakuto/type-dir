use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprRule, Quant};
use crate::walk::DirTree;
use crate::yaml::RuleName;

use super::super::candidate::{Candidate, GroupKey};
use super::super::expand::{ProducedOwner, ScopedEntry, expand_entries};
use super::super::memo::TrialMemo;
use super::eval_consume::eval_consume;
use super::eval_inner::{EvalContext, eval_entries, eval_node_chained};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::value::Record;

/// Processes an id-bearing Choice in a single descent and returns the Record list to push under choice_id.
///
/// Determines realization for each alternative (Record→trial, Own→name match), descends into and
/// consumes the realized alternative exactly once to finalize errors/dirs/produced/counts. Wraps the
/// produced from descent into a Record tagged with the alternative's id. The Choice entry itself is
/// pushed onto `all_expanded` and delegated to the parent-level choice realization (cardinality) check
/// (alt count is recorded in `counts`).
///
/// Child descent occurs at exactly one call site (`eval_node_traced` inside `eval_entries`/`eval_consume`);
/// the throwaway descent in trials is for realization determination only and is not the main traversal.
#[allow(clippy::too_many_arguments)] // choice evaluation genuinely requires all these parameters
pub(super) fn eval_choice(
    ctx: &mut EvalContext,
    group_id: &crate::yaml::EntryId,
    alternatives: &[ExprEntry],
    min: usize,
    max: Option<usize>,
    source_path: Option<String>,
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    let mut group_records: Vec<Record> = Vec::new();
    // For a multinode choice (one that contains Record alternatives), only the first realized alt is
    // pushed into produced (matching the one_of-style first-realized behavior of the old
    // collect_group_occurrences_multinode). Subsequent realized alts still descend/consume for
    // errors/dirs/counts (cardinality/choice realization checks), but are not pushed into
    // produced[choice_id]. As a result, produced[choice_id] for a multinode choice contains at most
    // 1 record (multiple only when an Own winner consumes several children). For a single-node
    // choice (all alternatives are Own), all realized alts are pushed into produced as in the old
    // single-node path (first-realized suppression is not applied).
    let has_multinode = !alternatives.iter().all(|alt| {
        matches!(
            alt.matcher,
            ExprMatcher::Dir { .. } | ExprMatcher::File { .. }
        )
    });
    let mut produced_emitted = false;
    // Push the Choice entry into all_expanded first and use its index as group_index. Even if
    // subtree-derived entries are appended later during alt processing, group_index remains fixed,
    // keeping the parent's choice realization check consistent with the recorded alt counts.
    let group_index = ctx.all_expanded.len();
    ctx.all_expanded.push((
        Cow::Owned(ExprEntry {
            id: None,
            source_path: source_path.clone(),
            count: Quant::Default,
            matcher: ExprMatcher::Choice {
                min,
                max,
                body: alternatives.to_vec(),
            },
        }),
        Cow::Owned(work_scope.clone()),
        None,
        ProducedOwner::Expanded,
    ));

    for (alt_idx, alt) in alternatives.iter().enumerate() {
        let tag = alt.id.as_ref().map(|id| id.0.clone());
        if matches!(
            alt.matcher,
            ExprMatcher::Dir { .. } | ExprMatcher::File { .. }
        ) {
            // single-node alternative: consume children using first-match and produce a tagged Record.
            let candidate = Candidate {
                entry: alt,
                scope: work_scope,
                group_key: Some(GroupKey {
                    group_index,
                    alt_index: alt_idx,
                }),
                origin: &None,
                entry_index: group_index,
            };
            let mut alt_sink = crate::runtime_impl::record_map::RecordMap::new();
            let consumed = eval_consume(ctx, &candidate, ProducedOwner::Expanded, &mut alt_sink);
            // For multinode, push only the first realized alt into produced (old first-realized behavior).
            // For single-node groups, push all realized alts. An Own winner may consume multiple children,
            // producing multiple Records only in that case.
            if !(consumed.is_empty() || has_multinode && produced_emitted) {
                for (captures, child_produced) in consumed {
                    group_records.push(Record {
                        fields: captures.into_iter().collect(),
                        children: child_produced
                            .into_iter()
                            .map(|(k, v)| (k, v.into_iter().map(Arc::new).collect()))
                            .collect(),
                        tag: tag.clone(),
                    });
                }
                produced_emitted = true;
            }
            continue;
        }

        // Non-Own alternative (Record / Splice / nested Choice / For / Match / Fetch / Bind):
        // trial-validates the content model `[alt]` for realization; if realized, descends at the
        // same node once and aggregates produced. Both realization check and descent delegate to
        // `eval_node`/`eval_entries` with `[alt]`, handling all kinds uniformly
        // (hermetic scope for Splice, binding expansion for For, and recursion for nested Choice
        // are all handled inside their respective paths).
        if !alternative_realizes(
            ctx.tree,
            std::slice::from_ref(alt),
            work_scope,
            ctx.rules,
            ctx.path,
            ctx.rule_name,
            ctx.rule_chain,
            ctx.memo,
        ) {
            continue;
        }
        // Record the alt count (for the parent's choice realization check).
        ctx.counts.record(&Candidate {
            entry: alt,
            scope: work_scope,
            group_key: Some(GroupKey {
                group_index,
                alt_index: alt_idx,
            }),
            origin: &None,
            entry_index: group_index,
        });
        // Descend into the content model at the same node once and aggregate produced. Push/pop a
        // frame to isolate inner ids from leaking into work_scope (only the wrapper id is bound by
        // the caller).
        work_scope.push();
        let mut alt_produced = crate::runtime_impl::record_map::RecordMap::new();
        eval_entries(
            ctx,
            std::slice::from_ref(alt),
            work_scope,
            &mut alt_produced,
        );
        work_scope.pop();
        // For multinode, push only the first realized alt into produced (old first-realized behavior).
        if !(has_multinode && produced_emitted) {
            group_records.push(Record {
                fields: IndexMap::new(),
                children: alt_produced
                    .into_iter()
                    .map(|(k, v)| (k, v.into_iter().map(Arc::new).collect()))
                    .collect(),
                tag,
            });
            produced_emitted = true;
        }
    }

    for rec in &group_records {
        produced
            .entry(group_id.0.clone())
            .or_default()
            .push(rec.clone());
    }
    work_scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Choice,
        group_id.0.clone(),
        group_records.clone(),
    );
}

/// Returns `true` unless every alternative of the Choice is a `Dir`/`File` matcher.
///
/// A choice whose alternatives are all `Dir`/`File` (single name patterns) is matched by the
/// conventional single-name assignment loop (`eval_consume`, which only handles `Dir`/`File`). Any
/// other alternative kind (Record / Splice / nested Choice / For / Match / Fetch / Bind) is a
/// multi-node content block that the single-name loop cannot represent, so the choice requires
/// trial-based realization and content-model expansion instead.
///
/// This generalization guarantees that, after `resolve_multinode_choices` / `eval_choice`,
/// `eval_consume` is only ever reached with `Dir`/`File` candidates: an all-`Dir`/`File` choice keeps
/// the single-node path, and every other choice is expanded away by the multi-node path. The
/// all-Use / no-sibling case is intercepted even earlier by `as_use_group` (content-choice).
pub(super) fn is_multinode_choice(entry: &ExprEntry) -> bool {
    match &entry.matcher {
        ExprMatcher::Choice { body, .. } => !body.iter().all(|alt| {
            matches!(
                alt.matcher,
                ExprMatcher::Dir { .. } | ExprMatcher::File { .. }
            )
        }),
        _ => false,
    }
}

/// Determines whether a multi-node alternative realizes for the given `tree`.
///
/// Runs `eval_node` on `subtree` entries and collects errors. An alternative realizes when all
/// resulting errors are `LintError::Undeclared`: these indicate nodes declared by sibling
/// alternatives or the outer scope, not failures within this alternative's own consumed nodes.
/// Any other error kind (e.g., `MissingRequired`, `CardinalityViolation`) causes the alternative
/// to not realize.
#[allow(clippy::too_many_arguments)]
pub(crate) fn alternative_realizes(
    tree: &DirTree,
    subtree: &[ExprEntry],
    scope: &Scope,
    rules: &IndexMap<RuleName, ExprRule>,
    path: &Path,
    rule_name: &str,
    rule_chain: &[String],
    memo: &mut TrialMemo,
) -> bool {
    let mut trial_errors: Vec<LintError> = Vec::new();
    let mut trial_produced = crate::runtime_impl::record_map::RecordMap::new();
    eval_node_chained(
        tree,
        subtree,
        scope,
        rules,
        path,
        rule_name,
        rule_chain,
        &mut trial_errors,
        &mut trial_produced,
        memo,
    );
    // Realize when no non-Undeclared error is found.
    trial_errors
        .iter()
        .all(|e| matches!(e, LintError::Undeclared { .. }))
}

/// Resolves multi-node choices in `expanded` by trial-based realization.
///
/// For each entry in `expanded`:
/// - If it is a multi-node Choice (any alternative is a `Record` matcher), trial-validates each
///   alternative. Records the realization count and:
///   - If the count is outside `[min, max]`, pushes `LintError::CardinalityViolation` and emits no
///     subtree entries (the error is the verdict; no partial expansion).
///   - Otherwise, inline-expands the realized alternatives' subtree entries via `expand_entries`
///     (all results are promoted to `Cow::Owned`) and appends them to the output in place of the
///     Choice entry.
///
///   For Own-alternative entries mixed into a multi-node choice, realization is determined by
///   whether any child of `tree` matches the alternative's name (c_a >= 1). This mirrors the
///   conventional `check_choice_realization` logic for single-node choices.
///
/// - All other entries (non-Choice, or Choice with only Own alternatives) pass through unchanged.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_multinode_choices<'a>(
    tree: &DirTree,
    expanded: Vec<ScopedEntry<'a>>,
    rules: &'a IndexMap<RuleName, ExprRule>,
    path: &Path,
    rule_name: &str,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
    memo: &mut TrialMemo,
) -> Vec<ScopedEntry<'a>> {
    let mut result: Vec<ScopedEntry<'a>> = Vec::with_capacity(expanded.len());

    for (entry_cow, scope_cow, origin, owner) in expanded {
        // Only process multi-node choices; all other entries pass through.
        if !is_multinode_choice(entry_cow.as_ref()) {
            result.push((entry_cow, scope_cow, origin, owner));
            continue;
        }

        // Extract min/max as copies so that we can release the borrow on entry_cow after
        // collecting the alternatives. Then alternatives are processed while entry_cow is alive.
        let (min, max) = match &entry_cow.as_ref().matcher {
            ExprMatcher::Choice { min, max, .. } => (*min, *max),
            _ => unreachable!("guarded by is_multinode_choice"),
        };
        let scope = scope_cow.as_ref();

        // Trial-validate each alternative and expand realized ones immediately.
        // All expansions are accumulated in realized_entries (Owned to avoid lifetime dependency
        // on the loop variable entry_cow / scope_cow).
        let mut realized = 0usize;
        let mut realized_entries: Vec<ScopedEntry<'a>> = Vec::new();
        if let ExprMatcher::Choice { body, .. } = &entry_cow.as_ref().matcher {
            for alt in body {
                // Realization is kind-independent: each alternative is its own content-model entry
                // list `[alt]`. `eval_node` (via `eval_entry`) handles every matcher kind
                // transparently — Own (name match), Record (subtree), Splice (hermetic expansion),
                // For (binding expansion), Match (winning arm), nested Choice (recursive), Fetch /
                // Bind (non-consuming, trivially realize). An alternative realizes when no non-
                // Undeclared error results.
                if !alternative_realizes(
                    tree,
                    std::slice::from_ref(alt),
                    scope,
                    rules,
                    path,
                    rule_name,
                    rule_chain,
                    memo,
                ) {
                    continue;
                }
                realized += 1;
                // Expand the realized alternative's content model. `expand_entries` resolves
                // For/Splice/Record/Match into Own/Choice entries (Bind/Fetch are consumed and emit
                // nothing here). Nested Choice entries are left intact by `expand_entries`, so the
                // result is recursively resolved: a nested multi-node choice is expanded away, while
                // a nested all-Own choice survives as a single-node Choice for the downstream
                // eval_consume path. This keeps eval_consume Own-only at every level.
                let sub_expanded = expand_entries(tree, std::slice::from_ref(alt), scope, rules);
                let sub_expanded = resolve_multinode_choices(
                    tree,
                    sub_expanded,
                    rules,
                    path,
                    rule_name,
                    rule_chain,
                    errors,
                    memo,
                );
                for (e, s, o, _) in sub_expanded {
                    realized_entries.push((
                        Cow::Owned(e.into_owned()),
                        Cow::Owned(s.into_owned()),
                        o,
                        ProducedOwner::Expanded,
                    ));
                }
            }
        }
        // The borrow of entry_cow via alternatives ends here.

        let below = realized < min;
        let above = max.is_some_and(|m| realized > m);

        if below || above {
            errors.push(LintError::CardinalityViolation {
                parent: path.to_path_buf(),
                realized,
                min,
                max,
                rule_chain: rule_chain.to_vec(),
                entry_path: entry_cow.as_ref().source_path.clone(),
            });
            // Do not emit any subtree entries when cardinality is violated.
        } else {
            result.extend(realized_entries);
        }
    }

    result
}
