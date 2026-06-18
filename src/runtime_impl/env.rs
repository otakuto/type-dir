#[cfg(test)]
#[path = "env_tests/tests.rs"]
mod tests;

use std::collections::HashMap;

use super::node_id::{NodeId, NodeKind, NodePath, NodePathElement};
use super::value::{Record, Value};

/// A single evaluation frame. Holds the lex bindings and env record sets introduced at this scope level.
///
/// - `lex` (Γ_lex): holds scalars/records originating from for bindings, with parameters, and
///   regex captures, keyed by `NodePathElement`.
/// - `env` (Γ_set): holds record sets overlaid by id producers, keyed by `NodePathElement`.
/// - `relaxed`: indicates whether this frame is in a relax context from an optional splice.
#[derive(Debug, Clone, Default, PartialEq)]
struct Frame {
    lex: HashMap<NodePathElement, Value>,
    env: HashMap<NodePathElement, Vec<Record>>,
    relaxed: bool,
}

/// The evaluation environment Γ as a frame stack.
///
/// get/set traverse the stack transparently (scanning all frames); push/pop only mark discard
/// intervals (fetch observation, backtrack trials, for instance isolation) — a conventional
/// language-runtime environment.
///
/// - lex/env are keyed by `NodePathElement { kind, id }`.
/// - bind always writes to the top frame.
/// - lookup scans from the top frame to the bottom frame (lexical shadowing).
/// - `get` is a kind-undetermined transparent accessor: scans all frames from top to bottom,
///   searching for an id match in each frame in lex (all kinds) → env (all kinds) order
///   (for bare references and regex capture passthrough).
///
/// # Roles of lex and env
///
/// - lex and env are separated by **value type**: lex holds scalar/record values (`Value`);
///   env holds record sets (`Vec<Record>`). The kind does **not** determine whether a binding
///   goes into lex or env.
/// - The same `(kind, id)` can coexist in both lex and env. Resolution is consistently
///   **lex-first → env fallback** (`get` / `resolve` / `head_records` / `head_value` all follow
///   this order).
/// - For example, with `for i in ... / id: i`, lex `(Value, "i")` (the iteration variable) and
///   env `(For, "i")` (the accumulated records) coexist without interfering. `${value.i}`
///   resolves to the former, `${for.i}` to the latter, and bare `${i}` resolves to the former
///   (lex-first).
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    frames: Vec<Frame>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    /// Creates a scope initialized with a single frame (empty, relaxed=false).
    pub fn new() -> Self {
        Self {
            frames: vec![Frame::default()],
        }
    }

    // ── Frame stack operations ─────────────────────────────────────────────────

    /// Pushes an empty frame (begins a discard interval).
    pub fn push(&mut self) {
        self.frames.push(Frame::default());
    }

    /// Pushes a frame with relaxed=true (begins the relax context of an optional splice).
    pub fn push_relaxed(&mut self) {
        self.frames.push(Frame {
            relaxed: true,
            ..Frame::default()
        });
    }

    /// Discards the top frame (ends the discard interval).
    pub fn pop(&mut self) {
        debug_assert!(self.frames.len() > 1, "pop must keep at least one frame");
        self.frames.pop();
    }

    /// Returns true if any frame is relaxed.
    pub fn relaxed(&self) -> bool {
        self.frames.iter().any(|f| f.relaxed)
    }

    /// Clears the relaxed flag on all frames (cuts off the relax context at child-node descent boundaries).
    ///
    /// The relax from an optional splice should only apply to body entries directly under the splice
    /// (i.e., at the splice's own node level); once descending into a materialized child directory,
    /// its contents must revert to required. Call this at the entry point of child-node evaluation
    /// to invalidate any relaxed frames that leaked into the incoming scope.
    pub fn clear_relaxed(&mut self) {
        for frame in &mut self.frames {
            frame.relaxed = false;
        }
    }

    // ── Bind operations (bind to the top frame) ──────────────────────────────────

    /// Binds a `Value` into Γ_lex (top frame) under the `(kind, id)` key.
    pub fn bind_lex(&mut self, kind: NodeKind, id: impl Into<NodeId>, value: Value) {
        self.top_mut()
            .lex
            .insert(NodePathElement::new(kind, id), value);
    }

    /// Merges a record set into Γ_set (top frame) under the `(kind, id)` key.
    ///
    /// If the same `(kind, id)` key already exists (in this frame or a lower frame), the records
    /// are accumulated by extending the existing contents (equivalent to the old `bind_records_into_scope`).
    pub fn bind_env(&mut self, kind: NodeKind, id: impl Into<NodeId>, records: Vec<Record>) {
        let key = NodePathElement::new(kind, id);
        let mut merged: Vec<Record> = self
            .lookup_env(kind, key.id.as_str())
            .map(<[Record]>::to_vec)
            .unwrap_or_default();
        merged.extend(records);
        self.top_mut().env.insert(key, merged);
    }

    /// Pre-declares (replaces) a record set in Γ_set (top frame) under the `(kind, id)` key.
    ///
    /// Used to overwrite pre-declared ids with an empty set so that same-named sets from outer scopes
    /// do not leak in.
    pub fn declare_env(&mut self, kind: NodeKind, id: impl Into<NodeId>, records: Vec<Record>) {
        self.top_mut()
            .env
            .insert(NodePathElement::new(kind, id), records);
    }

    // ── Lookup operations (scan from top frame to bottom frame) ─────────────────

    /// Looks up Γ_lex stack-transparently by `(kind, id)` key (id is compared as `&str`).
    pub fn lookup_lex(&self, kind: NodeKind, id: &str) -> Option<&Value> {
        let key = NodePathElement::new(kind, id);
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.lex.get(&key) {
                return Some(v);
            }
        }
        None
    }

    /// Looks up Γ_set stack-transparently by `(kind, id)` key (id is compared as `&str`).
    pub fn lookup_env(&self, kind: NodeKind, id: &str) -> Option<&[Record]> {
        let key = NodePathElement::new(kind, id);
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.env.get(&key) {
                return Some(v.as_slice());
            }
        }
        None
    }

    /// Kind-undetermined transparent accessor. Looks up by id alone for bare references and regex
    /// capture resolution.
    ///
    /// Scans all frames from top to bottom, searching for an id match within each frame in
    /// lex (all kinds) → env (all kinds) order. Returns the first match found (lex-first /
    /// env fallback, lexical shadowing). This allows bare passthrough to producer ids such as
    /// `with: q: ${id}` to resolve even when the id is on the env side.
    pub fn get(&self, id: &str) -> Option<ScopeRef<'_>> {
        for frame in self.frames.iter().rev() {
            for (key, v) in &frame.lex {
                if key.id.as_str() == id {
                    return Some(ScopeRef::Lex(v));
                }
            }
            for (key, records) in &frame.env {
                if key.id.as_str() == id {
                    return Some(ScopeRef::Set(records.as_slice()));
                }
            }
        }
        None
    }

    /// Resolves a qualified path (a `NodePath` like `dir.xxx.file.yyy`).
    ///
    /// - First segment: retrieves a record set via `head_records` (collects Record/RecordList from
    ///   lex; falls back to env if absent — lex-first → env fallback).
    /// - Subsequent segments: traverses `Record.children[id]` using the id key as-is (kind-blind).
    ///   Segments with kind `Regex` are treated as terminal field projections
    ///   (`fields[id]` → `ChainValue::Scalars`).
    ///
    /// Reuses `head_records` + `resolve_chain` internally. Returns `None` for empty paths.
    pub fn resolve(&self, path: &NodePath) -> Option<crate::runtime_impl::ref_resolve::ChainValue> {
        let (head, tail) = path.0.split_first()?;
        let recs = crate::runtime_impl::ref_resolve::head_records(
            self,
            crate::runtime_impl::ref_resolve::RefKind::Qualified(head.kind),
            head.id.as_str(),
        );
        // Convert tail segments to a `Hop` list (children use kind-blind id keys; Regex means field projection).
        let hops: Vec<crate::expr::Hop> = tail
            .iter()
            .map(|seg| match seg.kind {
                NodeKind::Regex => crate::expr::Hop::Regex(seg.id.0.clone()),
                _ => crate::expr::Hop::Dir(seg.id.0.clone()),
            })
            .collect();
        if hops.is_empty() {
            return Some(crate::runtime_impl::ref_resolve::ChainValue::Records(recs));
        }
        // Flat-map resolve_chain over the initial record set.
        let mut all_records: Vec<Record> = Vec::new();
        let mut all_scalars: Vec<String> = Vec::new();
        for rec in &recs {
            match crate::runtime_impl::ref_resolve::resolve_chain(rec, &hops) {
                Some(crate::runtime_impl::ref_resolve::ChainValue::Records(rs)) => {
                    all_records.extend(rs)
                }
                Some(crate::runtime_impl::ref_resolve::ChainValue::Scalars(ss)) => {
                    all_scalars.extend(ss)
                }
                None => {}
            }
        }
        if !all_scalars.is_empty() {
            Some(crate::runtime_impl::ref_resolve::ChainValue::Scalars(
                all_scalars,
            ))
        } else {
            Some(crate::runtime_impl::ref_resolve::ChainValue::Records(
                all_records,
            ))
        }
    }

    // ── Iterators (for fingerprinting) ────────────────────────────────────────────

    /// Iterates all Γ_lex bindings as (kind, id, value) (union of all frames, top frame wins).
    pub fn iter_lex(&self) -> impl Iterator<Item = (NodeKind, &str, &Value)> {
        let mut seen: HashMap<(NodeKind, &str), ()> = HashMap::new();
        let mut out: Vec<(NodeKind, &str, &Value)> = Vec::new();
        for frame in self.frames.iter().rev() {
            for (key, v) in &frame.lex {
                if seen.insert((key.kind, key.id.as_str()), ()).is_none() {
                    out.push((key.kind, key.id.as_str(), v));
                }
            }
        }
        out.into_iter()
    }

    /// Iterates all Γ_set record sets as (kind, id, records) (union of all frames, top frame wins).
    pub fn iter_sets(&self) -> impl Iterator<Item = (NodeKind, &str, &[Record])> {
        let mut seen: HashMap<(NodeKind, &str), ()> = HashMap::new();
        let mut out: Vec<(NodeKind, &str, &[Record])> = Vec::new();
        for frame in self.frames.iter().rev() {
            for (key, v) in &frame.env {
                if seen.insert((key.kind, key.id.as_str()), ()).is_none() {
                    out.push((key.kind, key.id.as_str(), v.as_slice()));
                }
            }
        }
        out.into_iter()
    }

    /// Returns a mutable reference to the top frame.
    fn top_mut(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("Scope always holds at least one frame")
    }
}

/// The reference returned by `Scope::get`. Distinguishes the lex side from the sets side.
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeRef<'a> {
    /// A value held by Γ_lex.
    Lex(&'a Value),
    /// A record set held by Γ_set (equivalent to `Value::RecordList`).
    Set(&'a [Record]),
}
