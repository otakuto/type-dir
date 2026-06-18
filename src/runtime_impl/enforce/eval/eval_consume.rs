use std::sync::Arc;

use indexmap::IndexMap;
use rayon::prelude::*;

use crate::expr::{ExprEntry, ExprMatcher, ExprSubtree};

use super::super::candidate::Candidate;
use super::super::expand::ProducedOwner;
use super::super::matcher::entry_matches_name;
use super::super::memo::TrialMemo;
use super::eval_inner::{EvalContext, eval_node_traced_chained};
use super::eval_shared::dominant_use_rule;
use crate::runtime_impl::value::Record;
use crate::runtime_impl::visible_ids::collect_visible_ids;

/// Consumes unconsumed child dirs/files for a single candidate using first-match.
///
/// Candidates are processed in source order, so all unconsumed children matching this candidate
/// are assigned to it (each child is assigned to the first candidate that matches it = first-match-wins).
/// Descends into each matched child exactly once via `eval_node_traced`, appending to the real
/// errors/dirs and receiving produced.
///
/// When `owner` is `Direct`, the id-bearing candidate acts as the producer and pushes a record into
/// `sink` (id-less candidates bubble child produced up transparently). When `owner` is `Expanded`,
/// the wrapper is the producer, so nothing is pushed into `sink`; instead, `(captures, child_produced)`
/// for each consumed child is returned for the caller to wrap (prevents double-counting).
///
/// Child descent occurs at exactly one call site (`eval_node_traced` inside this function) — dual
/// traversal has been eliminated.
pub(super) fn eval_consume(
    ctx: &mut EvalContext,
    candidate: &Candidate,
    _owner: ProducedOwner,
    sink: &mut crate::runtime_impl::record_map::RecordMap,
) -> Vec<(
    std::collections::HashMap<String, String>,
    crate::runtime_impl::record_map::RecordMap,
)> {
    let entry = candidate.entry;
    let (subtree, is_dir) = match &entry.matcher {
        ExprMatcher::Dir { subtree, .. } => (subtree, true),
        ExprMatcher::File { subtree, .. } => (subtree, false),
        _ => unreachable!("eval_consume only handles Dir/File candidates"),
    };
    let mut consumed: Vec<(
        std::collections::HashMap<String, String>,
        crate::runtime_impl::record_map::RecordMap,
    )> = Vec::new();

    if is_dir {
        // Determine whether the map-reduce path applies by inspecting the first Inline entry.
        // Only nodes with no id producers (collect_visible_ids returns empty) enter the map-reduce path.
        // Leaf nodes or nodes with non-empty visible_ids use the conventional sequential path.
        let use_map_reduce = matches!(subtree, ExprSubtree::Inline(inline) if collect_visible_ids(inline).is_empty());

        if use_map_reduce {
            // ── Phase 1: sequential consume phase ──
            // Perform name matching, bitset finalization, and counts.record sequentially, collecting
            // descent jobs into a Vec. eval_node_traced is not called yet.
            struct DescendJob {
                dir_idx: usize,
                captures: std::collections::HashMap<String, String>,
                child_path: std::path::PathBuf,
                child_scope: crate::runtime_impl::env::Scope,
                child_rule_owned: String,
                child_rule_chain: Vec<String>,
                inline_entries: Vec<crate::expr::ExprEntry>,
            }
            // When the number of jobs is below this threshold, run sequentially to avoid rayon fork/join overhead.
            const PAR_DESCEND_THRESHOLD: usize = 4;
            let mut jobs: Vec<DescendJob> = Vec::new();
            for (dir_idx, child_dir) in ctx.tree.dirs.iter().enumerate() {
                if ctx.consumed_dirs[dir_idx] {
                    continue;
                }
                let Some(captures) =
                    entry_matches_name(entry, &child_dir.name, candidate.scope, true)
                else {
                    continue;
                };
                ctx.consumed_dirs[dir_idx] = true;
                ctx.counts.record(candidate);

                let child_path = ctx.path.join(&child_dir.name);
                let ExprSubtree::Inline(inline) = subtree else {
                    unreachable!("use_map_reduce guarantees Inline subtree");
                };
                let inline_entries = inline.to_vec();
                let mut child_scope = candidate.scope.clone();
                if entry.id.is_some() {
                    for (k, v) in &captures {
                        child_scope.bind_lex(
                            crate::runtime_impl::node_id::NodeKind::Regex,
                            k.clone(),
                            crate::runtime_impl::value::Value::Scalar(v.clone()),
                        );
                    }
                }
                // Three-step rule for child rule_name: (a) if all inlines are bare splices to the same rule N, use N;
                // (b) otherwise fall back to the candidate's origin; (c) if neither applies, use the current rule_name.
                let child_rule_owned = dominant_use_rule(&inline_entries)
                    .or(candidate.origin.as_ref())
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| ctx.rule_name.to_string());
                // child rule_chain: append child_rule to the current chain (avoid tail duplicates).
                let child_rule_chain =
                    super::eval_shared::append_rule_chain(ctx.rule_chain, &child_rule_owned);
                jobs.push(DescendJob {
                    dir_idx,
                    captures,
                    child_path,
                    child_scope,
                    child_rule_owned,
                    child_rule_chain,
                    inline_entries,
                });
            }

            // ── Phase 2: descent phase ──
            // Map over the job Vec so each task produces results into an independent buffer.
            // A per-task TrialMemo is allocated (per-task is correct because siblings do not share memo hits).
            // into_par_iter().collect::<Vec<_>>() guarantees input-order = output-order, so the
            // subsequent Phase 3 merge is deterministic and matches the sequential version.
            struct DescendOut {
                errors: Vec<crate::error::LintError>,
                dirs: Vec<crate::runtime::DirTrace>,
                produced: crate::runtime_impl::record_map::RecordMap,
                captures: std::collections::HashMap<String, String>,
            }
            // Helper closure that executes the descent processing for each job.
            // Captures tree and rules as immutable references so they are Send+Sync and can be passed to rayon.
            // ctx is &mut EvalContext which is not Send, so only its immutable parts are copied locally.
            let tree = ctx.tree;
            let rules = ctx.rules;
            let run_job = |job: DescendJob| -> DescendOut {
                let mut task_errors = Vec::new();
                let mut task_dirs = Vec::new();
                let mut task_produced = crate::runtime_impl::record_map::RecordMap::new();
                let mut task_memo = TrialMemo::new();
                eval_node_traced_chained(
                    &tree.dirs[job.dir_idx],
                    &job.inline_entries,
                    &job.child_scope,
                    rules,
                    &job.child_path,
                    &job.child_rule_owned,
                    &job.child_rule_chain,
                    &mut task_errors,
                    &mut task_dirs,
                    &mut task_produced,
                    &mut task_memo,
                );
                DescendOut {
                    errors: task_errors,
                    dirs: task_dirs,
                    produced: task_produced,
                    captures: job.captures,
                }
            };
            let outs: Vec<DescendOut> = if jobs.len() < PAR_DESCEND_THRESHOLD {
                jobs.into_iter().map(run_job).collect()
            } else {
                jobs.into_par_iter().map(run_job).collect()
            };

            // ── Phase 3: sequential merge phase ──
            // Merge in job order (= source consume order) to guarantee byte-identical output with the sequential version.
            for out in outs {
                ctx.errors.extend(out.errors);
                ctx.dirs.extend(out.dirs);
                // Reflect produced: push a Record into sink for id-bearing entries; bubble child produced up transparently for id-less entries.
                if let Some(id) = &entry.id {
                    let record = Record {
                        fields: out.captures.clone().into_iter().collect(),
                        children: out
                            .produced
                            .clone()
                            .into_iter()
                            .map(|(k, v)| (k, v.into_iter().map(Arc::new).collect()))
                            .collect(),
                        tag: None,
                    };
                    sink.entry(id.0.clone()).or_default().push(record);
                }
                // Encapsulation: an id-less dir/file does NOT bubble its child produced to the
                // parent sink. Inner ids are hidden; cross-scope references must navigate through an
                // id-bearing ancestor's record (e.g. `${dir.<dirid>.file.<innerid>}`).
                consumed.push((out.captures, out.produced));
            }
        } else {
            // Conventional sequential path: visible_ids is non-empty or the subtree is a Leaf, so descend immediately.
            for (dir_idx, child_dir) in ctx.tree.dirs.iter().enumerate() {
                if ctx.consumed_dirs[dir_idx] {
                    continue;
                }
                let Some(captures) =
                    entry_matches_name(entry, &child_dir.name, candidate.scope, true)
                else {
                    continue;
                };
                ctx.consumed_dirs[dir_idx] = true;
                ctx.counts.record(candidate);

                let child_path = ctx.path.join(&child_dir.name);
                // Leaf directories are not inspected for contents (no descent into children). Only Inline subtrees descend as lexical nesting.
                let inline_entries: &[ExprEntry] = match subtree {
                    ExprSubtree::Inline(inline) => inline.as_slice(),
                    ExprSubtree::Leaf => {
                        // Contents are unchecked, but the consume is finalized. Produce an empty-children Record for id-bearing entries.
                        if let Some(id) = &entry.id {
                            let record = Record {
                                fields: captures.clone().into_iter().collect(),
                                children: IndexMap::new(),
                                tag: None,
                            };
                            sink.entry(id.0.clone()).or_default().push(record);
                        }
                        consumed
                            .push((captures, crate::runtime_impl::record_map::RecordMap::new()));
                        continue;
                    }
                };
                let mut child_scope = candidate.scope.clone();
                if entry.id.is_some() {
                    for (k, v) in &captures {
                        child_scope.bind_lex(
                            crate::runtime_impl::node_id::NodeKind::Regex,
                            k.clone(),
                            crate::runtime_impl::value::Value::Scalar(v.clone()),
                        );
                    }
                }
                // Three-step rule for child rule_name: (a) if all inlines are bare splices to the same rule N, use N;
                // (b) otherwise fall back to the candidate's origin; (c) if neither applies, use the current rule_name.
                let child_rule = dominant_use_rule(inline_entries)
                    .or(candidate.origin.as_ref())
                    .map(|n| n.0.as_str())
                    .unwrap_or(ctx.rule_name);
                // child rule_chain: append child_rule to the current chain (avoid tail duplicates).
                let child_rule_chain =
                    super::eval_shared::append_rule_chain(ctx.rule_chain, child_rule);

                // Descend into the child exactly once, append to the real errors/dirs, and receive child produced.
                let mut child_produced = crate::runtime_impl::record_map::RecordMap::new();
                eval_node_traced_chained(
                    child_dir,
                    inline_entries,
                    &child_scope,
                    ctx.rules,
                    &child_path,
                    child_rule,
                    &child_rule_chain,
                    ctx.errors,
                    ctx.dirs,
                    &mut child_produced,
                    ctx.memo,
                );

                // Reflect produced: push a Record into sink for id-bearing entries; for id-less entries,
                // bubble child produced up transparently into sink (rule A'). sink is provided by the caller
                // (parent produced or a wrapper-local sink). Wrappers (For/Record/Group) wrap via the return
                // value separately, so the sink is discarded.
                if let Some(id) = &entry.id {
                    let record = Record {
                        fields: captures.clone().into_iter().collect(),
                        children: child_produced
                            .clone()
                            .into_iter()
                            .map(|(k, v)| (k, v.into_iter().map(Arc::new).collect()))
                            .collect(),
                        tag: None,
                    };
                    sink.entry(id.0.clone()).or_default().push(record);
                }
                // Encapsulation: an id-less dir/file does NOT bubble its child produced to the
                // parent sink (inner ids stay hidden). Cross-scope references must navigate through
                // an id-bearing ancestor's record.
                consumed.push((captures, child_produced));
            }
        }
    } else {
        // ── file candidate: consume all unconsumed child files matching this candidate ──
        for (file_idx, file_name) in ctx.tree.files.iter().enumerate() {
            if ctx.consumed_files[file_idx] {
                continue;
            }
            let Some(captures) = entry_matches_name(entry, file_name, candidate.scope, false)
            else {
                continue;
            };
            ctx.consumed_files[file_idx] = true;
            ctx.counts.record(candidate);

            if let Some(id) = &entry.id {
                let record = Record {
                    fields: captures.clone().into_iter().collect(),
                    children: IndexMap::new(),
                    tag: None,
                };
                sink.entry(id.0.clone()).or_default().push(record);
            }
            consumed.push((captures, crate::runtime_impl::record_map::RecordMap::new()));
        }
    }
    consumed
}
