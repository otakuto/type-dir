use indexmap::IndexMap;

use crate::expr::Quant;

use super::eval_inner::{EvalContext, eval_entries};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::visible_ids::collect_visible_ids;

pub(super) fn eval_use(
    ctx: &mut EvalContext,
    rule: &crate::yaml::RuleName,
    with_args: &IndexMap<crate::yaml::VarName, String>,
    count: Quant,
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    if let Some(rule_def) = ctx.rules.get(rule) {
        let mut s_scope = super::super::with::build_with_scope(rule_def, with_args, work_scope);
        // Optional splice: when the use entry's count is relaxed (min=0), push a relaxed frame before
        // evaluating the rule body. This ensures the scope snapshot taken by required entries inside the
        // body has a relaxed frame on it, so the post-loop check will relax the required constraint.
        // For the normal case, just push a plain frame as a scope boundary.
        if count.is_relaxed() {
            s_scope.push_relaxed();
        } else {
            s_scope.push();
        }
        for (inner_kind, inner_id) in collect_visible_ids(&rule_def.rules) {
            s_scope.declare_env(inner_kind, inner_id, vec![]);
        }
        let kind_of = super::eval_shared::visible_kind_map(&rule_def.rules);
        let rule_str = rule.0.as_str();
        let mut use_produced = crate::runtime_impl::record_map::RecordMap::new();
        // Use inlines the rule body transparently at the current node, so the rule_chain is extended
        // with the use target name appended to the current chain (avoiding tail duplicates).
        let use_chain = super::eval_shared::append_rule_chain(ctx.rule_chain, rule_str);
        // Use uses a different rule_name than ctx.rule_name, so build a new EvalContext
        // with the overridden rule_name and reborrows of the mutable fields.
        let mut use_ctx = EvalContext {
            tree: ctx.tree,
            rules: ctx.rules,
            path: ctx.path,
            rule_name: rule_str,
            rule_chain: &use_chain,
            errors: &mut *ctx.errors,
            dirs: &mut *ctx.dirs,
            memo: &mut *ctx.memo,
            all_expanded: &mut *ctx.all_expanded,
            counts: &mut *ctx.counts,
            consumed_dirs: &mut *ctx.consumed_dirs,
            consumed_files: &mut *ctx.consumed_files,
        };
        eval_entries(
            &mut use_ctx,
            &rule_def.rules,
            &mut s_scope,
            &mut use_produced,
        );
        // explicit drop communicates intent to release the ctx borrow scope
        #[allow(clippy::drop_non_drop)]
        drop(use_ctx);
        // Discard the relax/normal frame (end of the discard interval).
        s_scope.pop();
        for (id, records) in use_produced {
            super::eval_shared::bind_produced_into_scope(work_scope, &kind_of, &id, &records);
            produced.entry(id).or_default().extend(records);
        }
    }
}
