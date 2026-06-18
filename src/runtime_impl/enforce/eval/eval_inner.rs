use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprRule, UseGroup, as_use_group};
use crate::runtime::DirTrace;
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::visible_ids::collect_visible_ids;
use crate::walk::DirTree;
use crate::yaml::RuleName;

use super::super::assignment_counts::AssignmentCounts;
use super::super::content_choice::eval_content_choice;
use super::super::expand::ScopedEntry;
use super::super::memo::TrialMemo;
use super::super::required::check_required_pairs;
use super::eval_choice::eval_choice;
use super::eval_shared::{check_choice_realization, check_interval_constraints};

/// Bundles the context and accumulators shared immutably across dispatch functions during
/// single-node evaluation. Node-local state (work_scope) and sink (produced aggregation target)
/// are swapped per call, so they remain individual arguments; only "carry-through for the full
/// node processing" data belongs here.
///
/// Borrow notes: tree/rules/path are immutable references; errors/dirs/memo/all_expanded/counts/
/// consumed_* are mutable references. During recursion (eval_consume -> eval_node_traced), a new
/// node-local state is created per child node, so ctx is not passed into recursion — a new
/// EvalContext is constructed at the entry of eval_node_traced.
pub(super) struct EvalContext<'a> {
    pub tree: &'a DirTree,
    pub rules: &'a IndexMap<RuleName, ExprRule>,
    pub path: &'a Path,
    pub rule_name: &'a str,
    /// The directory descent chain from the outside (outermost rule first). When generating errors,
    /// `build_effective_chain` appends the entry's splice origin and places it into `LintError.rule_chain`.
    pub rule_chain: &'a [String],
    pub errors: &'a mut Vec<LintError>,
    pub dirs: &'a mut Vec<DirTrace>,
    pub memo: &'a mut TrialMemo,
    pub all_expanded: &'a mut Vec<ScopedEntry<'static>>,
    pub counts: &'a mut AssignmentCounts,
    pub consumed_dirs: &'a mut [bool],
    pub consumed_files: &'a mut [bool],
}

/// Checks a single directory node (`tree`) against the given `entries` specification (Stage 2).
/// This is a pure evaluation function that does not access the file system.
///
/// A thin wrapper that does not collect dir traces. Called from trial runs where no trace should be
/// recorded (e.g. content-choice validation). Use `eval_node_traced` to record traces during actual checks.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_node(
    tree: &DirTree,
    entries: &[ExprEntry],
    scope: &Scope,
    rules: &IndexMap<RuleName, ExprRule>,
    path: &Path,
    rule_name: &str,
    errors: &mut Vec<LintError>,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
    memo: &mut TrialMemo,
) {
    // The chain seeds with the node's own rule name (callers without a chain context).
    let rule_chain = vec![rule_name.to_string()];
    eval_node_chained(
        tree,
        entries,
        scope,
        rules,
        path,
        rule_name,
        &rule_chain,
        errors,
        produced,
        memo,
    );
}

/// Same check as `eval_node`, but also traces the rule applied to each visited directory node into `dirs`.
///
/// Orchestrates the three terms of the acceptance condition in order: Cover ∧ interval constraint ∧ choice realization.
#[allow(clippy::too_many_arguments)]
pub fn eval_node_traced(
    tree: &DirTree,
    entries: &[ExprEntry],
    scope: &Scope,
    rules: &IndexMap<RuleName, ExprRule>,
    path: &Path,
    rule_name: &str,
    errors: &mut Vec<LintError>,
    dirs: &mut Vec<DirTrace>,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
    memo: &mut TrialMemo,
) {
    // The chain seeds with the node's own rule name (top-level entry point).
    let rule_chain = vec![rule_name.to_string()];
    eval_node_traced_chained(
        tree,
        entries,
        scope,
        rules,
        path,
        rule_name,
        &rule_chain,
        errors,
        dirs,
        produced,
        memo,
    );
}

/// `eval_node` with an explicit accumulated rule chain (internal descent path).
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_node_chained(
    tree: &DirTree,
    entries: &[ExprEntry],
    scope: &Scope,
    rules: &IndexMap<RuleName, ExprRule>,
    path: &Path,
    rule_name: &str,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
    memo: &mut TrialMemo,
) {
    // Discard the trace (for trials and dry-runs).
    let mut throwaway = Vec::new();
    eval_node_traced_chained(
        tree,
        entries,
        scope,
        rules,
        path,
        rule_name,
        rule_chain,
        errors,
        &mut throwaway,
        produced,
        memo,
    );
}

/// `eval_node_traced` with an explicit accumulated rule chain (internal descent path).
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_node_traced_chained(
    tree: &DirTree,
    entries: &[ExprEntry],
    scope: &Scope,
    rules: &IndexMap<RuleName, ExprRule>,
    path: &Path,
    rule_name: &str,
    rule_chain: &[String],
    errors: &mut Vec<LintError>,
    dirs: &mut Vec<DirTrace>,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
    memo: &mut TrialMemo,
) {
    // content-choice mode: when entries form a single Group whose alternatives are all Splices,
    // trial-validates the node's contents against each rule and evaluates the one_of/any_of constraint
    // using the count of valid alternatives. content-choice nodes are not expanded, so the trace is
    // recorded under the rule_name argument.
    if let Some(UseGroup {
        min,
        max,
        alternatives,
    }) = as_use_group(entries)
    {
        dirs.push(DirTrace {
            path: path.to_path_buf(),
            rule: rule_name.to_string(),
        });
        eval_content_choice(
            tree,
            alternatives,
            min,
            max,
            scope,
            rules,
            path,
            rule_name,
            rule_chain,
            errors,
            memo,
        );
        return;
    }

    // Single-pass traversal: expand entries one by one in source order, descend into consumed children
    // exactly once via eval_node_traced, and collect errors/dirs/produced simultaneously. Produced
    // from descent is incrementally reflected into the Γ_set of work_scope to make it visible for
    // for-source reference resolution of subsequent siblings (leveraging backward-only semantics).
    // Pre-declare all known ids with an empty set so same-named sets from outer scope do not leak in.
    let mut work_scope: Scope = scope.clone();
    // Child-node descent boundary: clear any relaxed frames that leaked into the incoming scope from
    // an optional splice. Relaxed frames are produced only by eval_use::push_relaxed, which is always
    // pushed inside the entry loop of this function (after this point). Therefore, any relaxed frame
    // present here in the incoming scope is always a "leak from child descent" and can safely be cleared.
    // The clear applies only to the cloned entry scope; it does not affect frames that subsequent
    // eval_use calls push via push_relaxed (e.g., scope snapshots for `- dir: tests` or
    // required-skip at the current node level).
    work_scope.clear_relaxed();
    for (kind, id) in collect_visible_ids(entries) {
        work_scope.declare_env(kind, id, vec![]);
    }

    // Record the rule applied to this node in the trace (recorded exactly once under the rule_name argument;
    // rule_name is correctly determined by the parent using the three-step rule before descending).
    dirs.push(DirTrace {
        path: path.to_path_buf(),
        rule: rule_name.to_string(),
    });

    // ── Single pass: expand + consume + child descent (produce and check simultaneously) ──
    // Accumulate all expanded entries in source order into all_expanded for the post-loop
    // parent-level checks (required / interval constraint / choice realization / undeclared).
    let mut all_expanded: Vec<ScopedEntry<'static>> = Vec::new();
    let mut counts = AssignmentCounts::new();
    let mut consumed_dirs: Vec<bool> = vec![false; tree.dirs.len()];
    let mut consumed_files: Vec<bool> = vec![false; tree.files.len()];

    {
        let mut ctx = EvalContext {
            tree,
            rules,
            path,
            rule_name,
            rule_chain,
            errors,
            dirs,
            memo,
            all_expanded: &mut all_expanded,
            counts: &mut counts,
            consumed_dirs: &mut consumed_dirs,
            consumed_files: &mut consumed_files,
        };
        for entry in entries {
            eval_entry(&mut ctx, entry, &mut work_scope, produced);
        }
    }

    // ── Undeclared: detect children not consumed by any candidate using deny-by-default ──
    for (dir_idx, child_dir) in tree.dirs.iter().enumerate() {
        if !consumed_dirs[dir_idx] {
            errors.push(LintError::Undeclared {
                path: path.join(&child_dir.name),
                is_dir: true,
                rule: rule_name.to_string(),
                rule_chain: rule_chain.to_vec(),
            });
        }
    }
    for (file_idx, file_name) in tree.files.iter().enumerate() {
        if !consumed_files[file_idx] {
            errors.push(LintError::Undeclared {
                path: path.join(file_name),
                is_dir: false,
                rule: rule_name.to_string(),
                rule_chain: rule_chain.to_vec(),
            });
        }
    }

    // ── required: checks required entries (entries with explicit count are excluded to consolidate into count judgment) ──
    check_required_pairs(tree, &all_expanded, path, rule_name, rule_chain, errors);

    // ── interval constraint: checks per-entry count constraints (checks the interval constraint term c_e∈[min,max]) ──
    check_interval_constraints(&all_expanded, &counts, path, rule_chain, errors);

    // ── choice realization: checks group realization count constraints (checks the choice realization term) ──
    check_choice_realization(&all_expanded, &counts, path, rule_chain, errors);
}

/// Processes a single source entry (raw entry) in source order.
///
/// Child descent into the subtree of this node occurs at exactly one call site: the
/// `eval_node_traced` call inside `eval_consume`. There is no separate produce-only descent, so
/// the subtree is descended into exactly once (dual-traversal has been eliminated).
///
/// - Bind: mutates work_scope and propagates to subsequent entries (produces no candidates).
/// - For/Record/Splice/Group(id-bearing): wrapper id producer. Descends into the body once via
///   `eval_consume`, aggregates the resulting produced into the wrapper's local sink, wraps it
///   into a Record, and binds the id.
/// - Own/Match/id-less wrapper: bubbles body produced directly up to the parent produced.
///
/// Expanded body candidates are accumulated in `all_expanded` in source order for the post-loop
/// parent-level checks (undeclared / required / interval constraint / choice realization).
/// `counts` is finalized at consume time.
pub(super) fn eval_entry(
    ctx: &mut EvalContext,
    entry: &ExprEntry,
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    // Value (value:) directly mutates work_scope and propagates to subsequent entries. Produces no candidates.
    if let ExprMatcher::Value { var, value } = &entry.matcher {
        super::eval_value::eval_value(var, value, work_scope);
        return;
    }

    // id-bearing Choice: performs the equivalent of collect_group_produced in a single descent.
    // The choice's body candidates go into cover/counts; produced from descent is wrapped into a
    // Record list under choice_id.
    if let (
        Some(group_id),
        ExprMatcher::Choice {
            body: alternatives,
            min,
            max,
        },
    ) = (&entry.id, &entry.matcher)
    {
        eval_choice(
            ctx,
            group_id,
            alternatives,
            *min,
            *max,
            entry.source_path.clone(),
            work_scope,
            produced,
        );
        return;
    }

    match &entry.matcher {
        ExprMatcher::For {
            var,
            source,
            body: for_rules,
        } => {
            super::eval_for::eval_for(
                ctx,
                entry.id.as_ref(),
                var,
                source,
                for_rules,
                work_scope,
                produced,
            );
        }
        ExprMatcher::Group { subtree } => {
            super::eval_group::eval_group(ctx, entry.id.as_ref(), subtree, work_scope, produced);
        }
        ExprMatcher::Use { rule, with_args } => {
            // Use transparency: processes the rule body in a hermetic scope and bubbles produced
            // up to the parent. Use cycles are eliminated at compile time, so no infinite recursion.
            // When entry.count is relaxed (optional), evaluate the body in a relax context to relax
            // inner required entries.
            super::eval_use::eval_use(ctx, rule, with_args, entry.count, work_scope, produced);
        }
        ExprMatcher::Match { scrutinee, arms } => {
            // Match: selects the winning arm by the scrutinee's tag and processes its subtree
            // transparently at the same node. Unbound / tag-less / no-matching-arm cases are
            // treated as no-expand (defensive behavior).
            super::eval_match::eval_match(ctx, scrutinee, arms, work_scope, produced);
        }
        ExprMatcher::Fetch { body: alts } => {
            // Fetch: observation-only and does not consume children, but reflects the id set of an
            // id-bearing Fetch into produced and work_scope (for for-source references of subsequent
            // siblings). expand_entries does not expand Fetch, so it is handled directly here.
            // An id-less Fetch is a no-op.
            if let Some(fetch_id) = &entry.id {
                super::eval_fetch::eval_fetch(ctx, &fetch_id.0, alts, work_scope, produced);
            }
        }
        _ => {
            // Dir / File / Group(id-less): consumes children as a leaf. Expands the body and
            // bubbles produced up to the parent (transparent producer).
            super::eval_dir_file::eval_dir_file(
                ctx,
                std::slice::from_ref(entry),
                work_scope,
                produced,
            );
        }
    }
}

/// Processes an inline-expanded entry list (body of for/record/splice/match) one entry at a time
/// in source order via `eval_entry`.
///
/// Each entry in the body is treated as an independent source entry, and nested wrapper ids
/// (For/Record/Group) are handled correctly. Produced is aggregated into `sink` (the parent
/// produced or a wrapper-local sink provided by the caller), and simultaneously reflected into
/// the Γ_set of `work_scope` at each entry processing step (for for-source visibility of subsequent
/// siblings).
pub(super) fn eval_entries(
    ctx: &mut EvalContext,
    entries: &[ExprEntry],
    work_scope: &mut Scope,
    sink: &mut crate::runtime_impl::record_map::RecordMap,
) {
    for entry in entries {
        eval_entry(ctx, entry, work_scope, sink);
    }
}
