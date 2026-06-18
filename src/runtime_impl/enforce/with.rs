use std::collections::HashMap;

use crate::expr::{ExprRule, parse_ref};
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::ref_resolve::{
    ChainValue, head_records, head_value, ref_head_parts, resolve_chain,
};
use crate::runtime_impl::template::resolve_template;
use crate::runtime_impl::value::Value;
use crate::yaml::VarName;
use indexmap::IndexMap;

/// Converts a `Scope` (Γ_lex) into a scalar-only `HashMap<String,String>`.
///
/// Only Γ_lex is traversed (Γ_set cannot be converted to scalars and is skipped).
/// - `Scalar` → `var → s`
/// - `Set` → skipped (cannot be determined as a single scalar value; the non-deterministic conversion
///   that silently takes the first element is removed, because `for` reads Set values directly from
///   the `Value` scope and iterates each element, so an approximation here is unnecessary).
/// - `Record` → `var → r.whole()` (main value = fields["0"], the full match). Additionally expands
///   `var.field → value` for bound record field references `${x.field}`.
/// - `RecordList` → skipped (cannot be converted to a scalar; references are resolved via `for` iteration).
pub fn scope_to_scalar(scope: &Scope) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (kind, k, v) in scope.iter_lex() {
        // Each lex binding is output as string keys matching the syntax by which templates can
        // reference them. Because `substitute` uses literal string matching, both the bare form
        // (`${x}`) and kind-qualified forms (`${value.x}` / `${with.x}` / `${rule.x}`) are emitted.
        // Value bindings can be referenced both as for-iteration variables (bare allowed) and as
        // value: bindings (`${value.x}`), so keys for both forms are registered.
        let mut keys: Vec<String> = vec![k.to_string()];
        match kind {
            NodeKind::Value => keys.push(format!("value.{k}")),
            NodeKind::With => keys.push(format!("with.{k}")),
            NodeKind::Rule => keys.push(format!("rule.{k}")),
            // Regex captures use bare form only. The env-side kind does not appear in lex traversal.
            _ => {}
        }
        match v {
            Value::Scalar(s) => {
                for key in &keys {
                    out.insert(key.clone(), s.clone());
                }
            }
            // Sets cannot be determined as a single scalar value, so skip them (the approximation of taking the first element is removed).
            Value::Set(_) => {}
            Value::Record(r) => {
                // Main value: `${x}` / `${value.x}` evaluates to the full match stored in fields["0"].
                for key in &keys {
                    out.insert(key.clone(), r.whole().to_string());
                    // For bound record field references `${x.field}` / `${value.x.field}`.
                    for (field, value) in &r.fields {
                        out.insert(format!("{key}.{field}"), value.clone());
                    }
                }
                // Child sets cannot be converted to scalars, so skip them.
            }
            // Record lists cannot be converted to scalars, so skip them (references are resolved via a different path).
            Value::RecordList(_) => {}
        }
    }
    out
}

/// Builds a hermetic child scope for descending into a rule reference.
///
/// Only the `with` params declared by the rule are visible, resolved from the caller's `with` args.
/// Required scalar params that are not supplied at the call site are left unbound.
/// No ambient lexical variables or id sets are inherited (fully hermetic).
///
/// Sets (`Value::RecordList`) can cross the boundary only through explicit with passing
/// (bare reference passthrough `with: q: ${id}`). The old implementation copied all RecordLists
/// from `outer_scope`, breaking hermeticity; however, the only beneficiary was the "self-owned ids of
/// the splice target rule", which are now collected and supplied at the time of splice expansion
/// (the splice arm in `expand.rs`). Moving ownership to the rule's own position establishes
/// compositionality, memoization soundness, and verification completeness.
pub fn build_with_scope(
    rule: &ExprRule,
    entry_with: &IndexMap<VarName, String>,
    outer_scope: &Scope,
) -> Scope {
    let mut s_scope = Scope::new();
    for (var_name, _shape) in &rule.with_params {
        if let Some(value) = entry_with.get(var_name) {
            let resolved = resolve_with_value(value, outer_scope);
            s_scope.bind_lex(NodeKind::With, var_name.0.clone(), resolved);
        }
    }
    s_scope
}

/// Returns the inner key string when `value` consists of exactly one `${...}` reference
/// (with no extra characters before or after). Returns `None` otherwise.
fn whole_single_ref(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("${")?.strip_suffix('}')?;
    // If the inner string contains another `}`, it is not a single reference, so exclude it.
    if inner.contains('}') {
        return None;
    }
    Some(inner)
}

/// Resolves a `with` argument value to a `Value` using the new namespace grammar.
///
/// - A plain string containing no `${...}` is always a literal → `Value::Scalar`.
/// - When the entire value is exactly one `${key}` reference, the `Value` is passed through as-is.
///   Head dispatch:
///   - `RefHead::Bare(name)` (rejected at compile) / `RefHead::WithNs { param }` /
///     `RefHead::RuleNs { rule_id }`: the lookup key is `name` / `param` / `rule_id` respectively.
///   - No hops (or empty tail): `scope.get(lookup_key)` → pass through as-is; `None` → empty `Set`.
///   - Qualified hops (Regex/Dir/File) in `hops` or the ns tail: the lookup key is resolved to
///     a record list, then `resolve_chain` is flat-mapped over each record.
///   - `Hop::Field` (unqualified) cannot appear here: compile-time `check_rule_var_scope` (E023)
///     rejects such references before configs reach the runtime.
///   - `RefHead::UseNs { id, tail }`: resolved on the same path as `RuleNs` — the splice instance
///     wrapper id is looked up and `tail` is followed via `resolve_chain`. The instance is referenced
///     directly; the desired record set is reached by explicit navigation (e.g. `.dir.<id>`).
/// - An embedded template (mixed with scalar literals) is resolved via `resolve_template`.
pub fn resolve_with_value(value: &str, scope: &Scope) -> Value {
    // A plain string containing no `${...}` is treated as a literal.
    if !value.contains("${") {
        return Value::Scalar(value.to_string());
    }

    // If the entire value is exactly one "${key}" reference, resolve the Value as-is.
    if let Some(key) = whole_single_ref(value) {
        let r = parse_ref(key);

        // Determine resolution kind, lookup id, and effective hops from the namespace head.
        // Value bindings live under `(NodeKind::Value, var)`.
        let (ref_kind, lookup_key_cow, effective_hops) = ref_head_parts(&r.head, &r.hops);
        let lookup_key: &str = &lookup_key_cow;

        if effective_hops.is_empty() {
            // No hops: resolve the head to a single value. Bare references use the transparent get
            // (lex → env); qualified heads use `(kind, id)` lookups. Return empty Set if not found.
            return head_value(scope, ref_kind, lookup_key).unwrap_or_else(|| Value::Set(vec![]));
        }

        // One or more qualified hops (Regex/Dir/File): resolve the head to a record list, then
        // flat-map resolve_chain over each record.
        let recs = head_records(scope, ref_kind, lookup_key);
        if recs.is_empty() {
            return Value::Set(vec![]);
        }
        let mut all_records = Vec::new();
        let mut all_scalars: Vec<String> = Vec::new();
        for rec in &recs {
            match resolve_chain(rec, effective_hops) {
                Some(ChainValue::Records(rs)) => all_records.extend(rs),
                Some(ChainValue::Scalars(ss)) => all_scalars.extend(ss),
                None => {}
            }
        }
        if !all_records.is_empty() {
            return Value::RecordList(all_records);
        }
        if let [single] = all_scalars.as_slice() {
            return Value::Scalar(single.clone());
        }
        return Value::Set(vec![]);
    }

    // Embedded template (mixed with scalars) → resolve to a single scalar via resolve_template.
    Value::Scalar(resolve_template(value, scope))
}
