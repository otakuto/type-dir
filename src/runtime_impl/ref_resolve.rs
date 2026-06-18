#[cfg(test)]
#[path = "ref_resolve_tests/tests.rs"]
mod tests;

use std::borrow::Cow;

use crate::expr::{Hop, RefHead};
use crate::runtime_impl::env::{Scope, ScopeRef};
use crate::runtime_impl::node_id::{NodeKind, NodePath, NodePathElement};
use crate::runtime_impl::value::{Record, Value};

/// The result of resolving a full hop chain.
#[derive(Debug, PartialEq, Eq)]
pub enum ChainValue {
    /// Chain ended in a scalar leaf (Regex). May contain 0, 1, or many values
    /// (many when a record set was crossed by intermediate dir/file hops).
    Scalars(Vec<String>),
    /// Chain ended in a dir/file hop (a record set).
    Records(Vec<Record>),
}

/// The resolution method for a reference head.
///
/// - `Bare`: a bare reference with undetermined kind. Looked up by id alone via the transparent
///   get (`Scope::get`).
/// - `Qualified(kind)`: a kind-qualified reference. Looked up via `lookup_lex(kind, id)` /
///   `lookup_env(kind, id)` / `head_records(scope, kind, id)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Bare,
    Qualified(NodeKind),
}

/// Extracts the resolution kind, lookup key, and effective hops from a `RefHead`.
///
/// Returns `(RefKind, Cow<str>, &[Hop])`. The `Cow` is `Borrowed(id)` for every variant.
///
/// - `Bare(name)` → `(RefKind::Bare, name)`; hops are the top-level `hops` slice. Bare references are
///   kind-undetermined, so callers resolve them via the transparent `Scope::get(id)` accessor (not
///   `lookup_lex`), so producer-id passthrough (`with: q: ${id}`) on the env side resolves.
/// - `WithNs { param, tail }` → `(Qualified(With), param)`; hops are `tail`.
/// - `RuleNs { rule_id, tail }` → `(Qualified(Rule), rule_id)`; hops are `tail`.
/// - `ValueNs { var, tail }` → `(Qualified(Value), var)`; hops are `tail`. The legacy `"value."` prefix
///   key is gone — value bindings live under `(NodeKind::Value, var)`.
/// - `UseNs { id, tail }` → `(Qualified(Group), id)`; hops are `tail`. A splice with an id
///   (`- use: rule.R / id: X`) desugars at compile time to a `Group{ id: X, subtree: [use] }`, so
///   the splice instance wrapper is a Group producer; `${use.X}` and `${group.X}` resolve identically.
/// - `ForNs { id, tail }` → `(Qualified(For), id)`; hops are `tail`.
/// - `FetchNs { id, tail }` → `(Qualified(Fetch), id)`; hops are `tail`.
/// - `DirNs { id, tail }` → `(Qualified(Dir), id)`; hops are `tail`.
/// - `FileNs { id, tail }` → `(Qualified(File), id)`; hops are `tail`.
/// - `GroupNs { id, tail }` → `(Qualified(Group), id)`; hops are `tail`.
/// - `ChoiceNs { id, tail }` → `(Qualified(Choice), id)`; hops are `tail`.
pub fn ref_head_parts<'a>(
    head: &'a RefHead,
    hops: &'a [Hop],
) -> (RefKind, Cow<'a, str>, &'a [Hop]) {
    match head {
        RefHead::Bare(name) => (RefKind::Bare, Cow::Borrowed(name.as_str()), hops),
        RefHead::WithNs { param, tail } => (
            RefKind::Qualified(NodeKind::With),
            Cow::Borrowed(param.as_str()),
            tail.as_slice(),
        ),
        RefHead::RuleNs { rule_id, tail } => (
            RefKind::Qualified(NodeKind::Rule),
            Cow::Borrowed(rule_id.as_str()),
            tail.as_slice(),
        ),
        RefHead::ValueNs { var, tail } => (
            RefKind::Qualified(NodeKind::Value),
            Cow::Borrowed(var.as_str()),
            tail.as_slice(),
        ),
        RefHead::UseNs { id, tail } => (
            // splice+id desugars to Group at compile time, so the Use namespace also resolves to Group kind.
            RefKind::Qualified(NodeKind::Group),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
        RefHead::ForNs { id, tail } => (
            RefKind::Qualified(NodeKind::For),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
        RefHead::FetchNs { id, tail } => (
            RefKind::Qualified(NodeKind::Fetch),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
        RefHead::DirNs { id, tail } => (
            RefKind::Qualified(NodeKind::Dir),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
        RefHead::FileNs { id, tail } => (
            RefKind::Qualified(NodeKind::File),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
        RefHead::GroupNs { id, tail } => (
            RefKind::Qualified(NodeKind::Group),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
        RefHead::ChoiceNs { id, tail } => (
            RefKind::Qualified(NodeKind::Choice),
            Cow::Borrowed(id.as_str()),
            tail.as_slice(),
        ),
    }
}

/// Resolves a `RefKind` + id to a single bound `Value`, for scalar / single-value contexts.
///
/// - `RefKind::Bare` → transparent `Scope::get(id)`; only the `Γ_lex` side yields a `Value`
///   (the `Γ_set` side is returned as `RecordList` so callers can branch uniformly).
/// - `RefKind::Qualified(kind)` → `lookup_lex(kind, id)` (cloned), falling back to `lookup_env`
///   as a `RecordList` when the lex side is absent.
pub fn head_value(scope: &Scope, ref_kind: RefKind, id: &str) -> Option<Value> {
    match ref_kind {
        RefKind::Bare => scope.get(id).map(|r| match r {
            ScopeRef::Lex(v) => v.clone(),
            ScopeRef::Set(recs) => Value::RecordList(recs.to_vec()),
        }),
        RefKind::Qualified(kind) => match scope.lookup_lex(kind, id) {
            Some(v) => Some(v.clone()),
            None => scope
                .lookup_env(kind, id)
                .map(|recs| Value::RecordList(recs.to_vec())),
        },
    }
}

/// Builds a `NodePath` from a `Qualified` head and its tail hops.
///
/// The first segment is the head's `(kind, id)`. Each tail hop is mapped to a `NodePathElement`:
/// producer hops (Dir/File/Choice/Group/For/Fetch) become kind-blind segments that traverse children
/// by id key (represented by `NodeKind::Dir`); `Regex(g)` becomes a terminal field-projection segment
/// (`NodeKind::Regex`). `Bare` heads and `Hop::Field` cannot be expressed as a `NodePath` (callers
/// handle them via transparent get / existing paths), so `None` is returned.
pub fn node_path_from_ref(ref_kind: RefKind, id: &str, hops: &[Hop]) -> Option<NodePath> {
    let RefKind::Qualified(kind) = ref_kind else {
        return None;
    };
    let mut elements = vec![NodePathElement::new(kind, id)];
    for hop in hops {
        let element = match hop {
            Hop::Regex(g) => NodePathElement::new(NodeKind::Regex, g.as_str()),
            Hop::Dir(id)
            | Hop::File(id)
            | Hop::Choice(id)
            | Hop::Group(id)
            | Hop::For(id)
            | Hop::Fetch(id) => NodePathElement::new(NodeKind::Dir, id.as_str()),
            // Legacy field hops cannot be handled in a chain, so they are not expressed as NodePath.
            Hop::Field(_) => return None,
        };
        elements.push(element);
    }
    Some(NodePath(elements))
}

/// Resolves a `RefKind` + id to a flat list of `Record` values, consulting both `Γ_lex` and `Γ_set`.
///
/// - `RefKind::Bare` → transparent `Scope::get(id)`:
///   - `Γ_lex` / `Value::Record(r)` → `[r]`
///   - `Γ_lex` / `Value::RecordList(recs)` → each record
///   - `Γ_set` → each record
/// - `RefKind::Qualified(kind)` → `lookup_lex(kind, id)` (Record / RecordList) then
///   `lookup_env(kind, id)` (record set), in that priority order.
///
/// Returns an empty `Vec` when the key is absent or holds a non-record value (`Scalar`/`Set`).
/// This is the correct exhaustive record-gathering behaviour shared by `resolve_with_value`
/// (with.rs) and `resolve_expr_source` (expand.rs).
pub fn head_records(scope: &Scope, ref_kind: RefKind, id: &str) -> Vec<Record> {
    match ref_kind {
        RefKind::Bare => match scope.get(id) {
            Some(ScopeRef::Lex(Value::Record(rec))) => vec![rec.clone()],
            Some(ScopeRef::Lex(Value::RecordList(recs))) => recs.clone(),
            Some(ScopeRef::Set(recs)) => recs.to_vec(),
            _ => vec![],
        },
        RefKind::Qualified(kind) => match scope.lookup_lex(kind, id) {
            Some(Value::Record(rec)) => vec![rec.clone()],
            Some(Value::RecordList(recs)) => recs.clone(),
            _ => scope
                .lookup_env(kind, id)
                .map(<[Record]>::to_vec)
                .unwrap_or_default(),
        },
    }
}

/// Resolves a full **qualified** hop chain starting from a single record.
///
/// Intermediate `Dir`/`File` hops descend into children, flat-mapping over record sets
/// (map+flatten). Terminal `Regex` hops project to scalars over the current set.
/// Returns `None` if a `Hop::Field` (legacy) appears anywhere in the chain — legacy references
/// are handled by callers as single-hop special cases and are never composed into chains.
///
/// # Semantics
///
/// - `Dir(id)` / `File(id)`: descend into `children[id]` for every record in the current set,
///   flat-mapping the results (kind-blind at runtime).
/// - `Regex(g)`: project to `fields[g]` for each record that has it → `Scalars`.
///   Use `"0"` for the full match, `"1"`, `"2"`, ... for positional groups, or a name for named groups.
/// - `Field(_)`: unsupported in chain context → `None`.
///
/// When hops is empty the caller is responsible for not calling this function (bare references
/// are handled separately). An empty slice produces `Records([start])`.
pub fn resolve_chain(start: &Record, hops: &[Hop]) -> Option<ChainValue> {
    let mut recs: Vec<Record> = vec![start.clone()];
    for hop in hops {
        match hop {
            // All producer hops navigate into the child record set keyed by `id` (kind-blind at
            // runtime; the kind label is used only for static validation).
            Hop::Dir(id)
            | Hop::File(id)
            | Hop::Choice(id)
            | Hop::Group(id)
            | Hop::For(id)
            | Hop::Fetch(id) => {
                recs = recs
                    .iter()
                    .flat_map(|r| {
                        r.children
                            .get(id)
                            .map(|v| v.iter().map(|rc| (**rc).clone()).collect::<Vec<_>>())
                            .unwrap_or_default()
                    })
                    .collect();
            }
            Hop::Regex(g) => {
                return Some(ChainValue::Scalars(
                    recs.iter()
                        .filter_map(|r| r.fields.get(g.as_str()).cloned())
                        .collect(),
                ));
            }
            Hop::Field(_) => return None,
        }
    }
    // Chain ended on dir/file hops (or empty slice) → a record set.
    Some(ChainValue::Records(recs))
}
