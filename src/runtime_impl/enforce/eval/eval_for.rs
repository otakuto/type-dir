use std::sync::Arc;

use indexmap::IndexMap;

use crate::expr::{ExprEntry, ExprForSource};

use super::super::expand::{ForBinding, resolve_for_source};
use super::eval_inner::{EvalContext, eval_entries};
use super::eval_shared::{bind_produced_into_scope, lift_tag_from_child_out, visible_kind_map};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::{Record, Value};
use crate::runtime_impl::visible_ids::collect_visible_ids;

pub(super) fn eval_for(
    ctx: &mut EvalContext,
    for_id: Option<&crate::yaml::EntryId>,
    var: &crate::yaml::VarName,
    source: &ExprForSource,
    for_rules: &[ExprEntry],
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    let bindings = resolve_for_source(source, work_scope);
    let mut for_records: Vec<Record> = Vec::new();
    for binding in bindings {
        // Push a frame onto work_scope for each binding (instance boundary); bindings within this
        // binding are reflected only into this frame. Pop and discard the frame at the end of the
        // binding (nothing leaks to the parent = instance isolation).
        work_scope.push();
        // The iteration variable is bound under `(NodeKind::Value, itervar)` so it can be referenced
        // from the body via `${value.<itervar>}`.
        let binding_fields: IndexMap<String, String> = match &binding {
            ForBinding::Scalar(s) => {
                work_scope.bind_lex(NodeKind::Value, var.0.clone(), Value::Scalar(s.clone()));
                IndexMap::new()
            }
            ForBinding::Record(r) => {
                work_scope.bind_lex(NodeKind::Value, var.0.clone(), Value::Record(r.clone()));
                r.fields.clone()
            }
        };
        for (inner_kind, inner_id) in collect_visible_ids(for_rules) {
            work_scope.declare_env(inner_kind, inner_id, vec![]);
        }
        let kind_of = visible_kind_map(for_rules);
        let mut binding_produced = crate::runtime_impl::record_map::RecordMap::new();
        eval_entries(ctx, for_rules, work_scope, &mut binding_produced);
        work_scope.pop();
        if for_id.is_some() {
            let lifted_tag = lift_tag_from_child_out(&binding_produced);
            for_records.push(Record {
                fields: binding_fields,
                children: binding_produced
                    .into_iter()
                    .map(|(k, v)| (k, v.into_iter().map(Arc::new).collect()))
                    .collect(),
                tag: lifted_tag,
            });
        } else {
            // id-less For: processes the body transparently for each binding and bubbles produced
            // up to the parent. Each binding is an instance boundary (cloned scope), but the union
            // of produced is reflected into the parent.
            for (id, records) in binding_produced {
                bind_produced_into_scope(work_scope, &kind_of, &id, &records);
                produced.entry(id).or_default().extend(records);
            }
        }
    }
    if let Some(for_id) = for_id {
        for rec in &for_records {
            produced
                .entry(for_id.0.clone())
                .or_default()
                .push(rec.clone());
        }
        work_scope.bind_env(NodeKind::For, for_id.0.clone(), for_records.clone());
    }
}
