use std::sync::Arc;

use indexmap::IndexMap;

/// A record corresponding to a node matched by one id entry.
///
/// - `fields`: captures from this entry only. `fields["0"]` is the full match (the record's main
///   value; bare `${x}` evaluates to this). Positional groups are stored as `"1"`, `"2"`, etc.
///   Named groups are stored under their declared name.
/// - `children`: nested records keyed by child id. Each child is reference-counted so that
///   `Record::clone` performs a shallow copy (Arc::clone) rather than recursively deep-copying
///   the entire subtree, avoiding O(n^2) cloning of deep trees.
/// - `tag`: first-match winner alt id when this record was produced by an id-bearing Group
///   (Sum occurrence). `None` for records not derived from a Sum (Own dir/file, Record-intro).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    pub fields: IndexMap<String, String>,
    pub children: IndexMap<String, Vec<Arc<Record>>>,
    pub tag: Option<String>,
}

impl Record {
    /// Returns the full match string (the record's main value), i.e. `fields["0"]`.
    ///
    /// Returns an empty string if `fields["0"]` is absent.
    pub fn whole(&self) -> &str {
        self.fields.get("0").map(String::as_str).unwrap_or("")
    }
}

/// Represents a variable value (scalar, set, record, or record list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A single-string scalar value.
    Scalar(String),
    /// A multi-string set value (order-preserving, no duplicates).
    Set(Vec<String>),
    /// A record value (used when binding a record to a variable in a for-iteration).
    Record(Record),
    /// A list of records collected by an id producer at the same node.
    /// Passes through Γ_lex only via with-passthrough (`with: q: ${id}` bare reference).
    /// Use `Scope::bind_env` / `Scope::get` for registration and lookup in Γ_set.
    RecordList(Vec<Record>),
}
