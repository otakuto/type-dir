use crate::expr::{MatchArm, parse_ref};

use super::eval_inner::{EvalContext, eval_entries};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::ref_resolve::{head_value, ref_head_parts};
use crate::runtime_impl::value::Value;

pub(super) fn eval_match(
    ctx: &mut EvalContext,
    scrutinee: &str,
    arms: &[MatchArm],
    work_scope: &mut Scope,
    produced: &mut crate::runtime_impl::record_map::RecordMap,
) {
    // scrutinee is the inner key inside `${...}` (e.g. `value.record`). Parse it with the namespace
    // grammar, resolve to (kind, id), and look up the Record. A bare head is resolved via transparent get.
    let r = parse_ref(scrutinee);
    let (ref_kind, id_cow, _hops) = ref_head_parts(&r.head, &r.hops);
    if let Some(Value::Record(rec)) = head_value(work_scope, ref_kind, &id_cow) {
        let tag = rec.tag.clone();
        if let Some(arm) = arms
            .iter()
            .find(|a| Some(a.tag.0.as_str()) == tag.as_deref())
        {
            eval_entries(ctx, &arm.subtree, work_scope, produced);
        }
    }
}
