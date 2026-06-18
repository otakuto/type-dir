use std::sync::Arc;

use indexmap::IndexMap;

use crate::expr::ExprEntry;

use super::eval_inner::{EvalContext, eval_entries};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::Record;

pub(super) fn eval_group(
    ctx: &mut EvalContext,
    id: Option<&crate::yaml::EntryId>,
    subtree: &[ExprEntry],
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    if let Some(id) = id {
        // id-bearing Record-intro: descends into the subtree once at the same node and wraps produced under the id.
        // Pushes a frame to isolate id bindings of the subtree, then pops and discards it after wrapping
        // (only the wrapper id is bound into the caller's parent).
        work_scope.push();
        let mut sub_produced = crate::runtime_impl::record_map::RecordMap::new();
        eval_entries(ctx, subtree, work_scope, &mut sub_produced);
        work_scope.pop();
        let record = Record {
            fields: IndexMap::new(),
            children: sub_produced
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(Arc::new).collect()))
                .collect(),
            tag: None,
        };
        produced
            .entry(id.0.clone())
            .or_default()
            .push(record.clone());
        work_scope.bind_env(NodeKind::Group, id.0.clone(), vec![record.clone()]);
    } else {
        // id-less Record-intro: processes the subtree transparently at the same node and bubbles produced up to the parent.
        eval_entries(ctx, subtree, work_scope, produced);
    }
}
