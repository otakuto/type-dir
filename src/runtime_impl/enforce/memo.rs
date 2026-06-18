use std::collections::HashMap;
use std::fmt::Write as _;

use crate::error::LintError;
use crate::runtime_impl::env::Scope;
use crate::runtime_impl::value::{Record, Value};
use crate::walk::DirTree;
use crate::yaml::RuleName;

/// Memoization key for a content-choice trial: `(node identity, rule name, σ fingerprint)`.
///
/// - Node identity: the stable pointer value of the `DirTree` node being traversed (see `memo_node_id`).
/// - Rule name: the rule of the alternative used in the trial.
/// - σ fingerprint: the effective scope passed to the trial, serialized in a stable order.
pub type TrialKey = (usize, RuleName, String);

/// Memoization table for content-choice trial evaluations. Reuses trial error lists per `(node, rule, σ)`.
///
/// In nested content-choices, the same `(node, rule, scope)` trial may be re-executed exponentially
/// many times. Because improvement #2 made rule semantics a function only of the hermetic scope
/// (compositionality), memoization by `(node, rule, σ)` is sound. The `dirs` trace within a trial is
/// discarded and is not included in the result; only `errors` are cached.
#[derive(Default)]
pub struct TrialMemo {
    map: HashMap<TrialKey, Vec<LintError>>,
}

impl TrialMemo {
    /// Creates an empty memo table.
    pub fn new() -> TrialMemo {
        Self::default()
    }

    /// Returns a borrowed reference to the cached trial error list for the given key, if present.
    pub fn get(&self, key: &TrialKey) -> Option<&Vec<LintError>> {
        self.map.get(key)
    }

    /// Stores the trial error list under the given key.
    pub fn insert(&mut self, key: TrialKey, errors: Vec<LintError>) {
        self.map.insert(key, errors);
    }
}

/// Returns the stable identity of a `DirTree` node.
///
/// Within a single `check_dir` traversal, each node is referenced via a fixed `&DirTree` allocation,
/// so the pointer value is unique and stable for the duration of the traversal and can be used as a
/// node identity. Because it is not retained across traversals, collisions from re-allocation cannot occur.
pub fn memo_node_id(tree: &DirTree) -> usize {
    tree as *const DirTree as usize
}

/// Serializes scope σ into a stable fingerprint string.
///
/// Because `HashMap` iteration order is non-deterministic, keys are sorted in ascending order before
/// serialization so that the same scope always maps to the same string. Γ_lex and Γ_set are
/// distinguished by "lex:" / "set:" prefixes before sorting and serialization.
/// `Value` is serialized recursively; `Record` is stabilized in name → fields → children order.
pub fn fingerprint_scope(scope: &Scope) -> String {
    // Merge lex and env keys as "lex:<kind>:<id>" / "set:<kind>:<id>" and sort in ascending order.
    // Including the kind keeps disjoint-kind entries (e.g. value.i vs for.i) distinct, and the sort
    // makes the fingerprint deterministic regardless of HashMap iteration order.
    let mut pairs: Vec<(String, FpKind<'_>)> = Vec::new();
    for (kind, k, v) in scope.iter_lex() {
        pairs.push((format!("lex:{kind:?}:{k}"), FpKind::Lex(v)));
    }
    for (kind, k, records) in scope.iter_sets() {
        pairs.push((format!("set:{kind:?}:{k}"), FpKind::Set(records)));
    }
    pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::new();
    for (key, kind) in pairs {
        let _ = write!(out, "{key}=");
        match kind {
            FpKind::Lex(v) => write_value(&mut out, v),
            FpKind::Set(records) => write_record_list(&mut out, records),
        }
        out.push(';');
    }
    out
}

/// Value kind used internally by `fingerprint_scope`.
enum FpKind<'a> {
    Lex(&'a Value),
    Set(&'a [Record]),
}

/// Appends a stable serialization of a single `Value` to the buffer.
fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Scalar(s) => {
            out.push_str("S(");
            out.push_str(s);
            out.push(')');
        }
        Value::Set(items) => {
            // Sets are declared to be order-preserving, so serialize them as-is.
            out.push_str("L[");
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(item);
            }
            out.push(']');
        }
        Value::Record(record) => write_record(out, record),
        Value::RecordList(records) => write_record_list(out, records),
    }
}

fn write_record_list(out: &mut String, records: &[Record]) {
    out.push_str("RL[");
    for (idx, record) in records.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        write_record(out, record);
    }
    out.push(']');
}

/// Appends a stable serialization of a `Record` to the buffer in fields → children order.
///
/// `fields` and `children` are `IndexMap`, which preserves insertion order, so serializing them
/// as-is produces a stable order. `fields["0"]` is the full match (the record's main value).
/// When `tag` is `Some(t)`, a `t=<t>,` prefix is prepended before the fields to distinguish
/// Sum-tagged records. `None` produces no prefix, preserving pre-stage-6 fingerprints exactly.
fn write_record(out: &mut String, record: &Record) {
    out.push_str("R{f=");
    if let Some(t) = &record.tag {
        let _ = write!(out, "t={t},");
    }
    for (field, value) in &record.fields {
        let _ = write!(out, "{field}:{value},");
    }
    out.push_str(";c=");
    for (child_id, children) in &record.children {
        let _ = write!(out, "{child_id}:[");
        for (idx, child) in children.iter().enumerate() {
            if idx > 0 {
                out.push(',');
            }
            write_record(out, child);
        }
        out.push(']');
    }
    out.push('}');
}
