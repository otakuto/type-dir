/// Domain ID newtype representing the id portion of scope lex/env keys.
///
/// Paired with `NodeKind` and used as scope keys in the form `NodePathElement { kind, id }`.
/// Raw `String` ids from various sources (EntryId / RuleName / regex capture names / var names)
/// are converted to this type at bind sites. `Hash`/`Eq` are required for use as HashMap keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl NodeId {
    /// Borrows the inner string as `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        NodeId(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId(s)
    }
}

/// The kind representing all namespaces of the reference grammar. Used as keys in `NodePathElement`
/// for both the lex side and the env side.
///
/// The kind does not determine whether a binding goes into lex or env (that is determined by value type).
/// The same `(kind, id)` can coexist in both lex and env maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    // kinds primarily used on the env (record-set producer) side
    Dir,
    File,
    For,
    Group,
    Choice,
    Fetch,
    Use,
    // kinds primarily used on the lex (scalar/record binding) side
    With,
    Rule,
    Value,
    /// lex kind that exposes regex capture groups to children under bare names.
    Regex,
}

/// A single segment of a reference path. Serves as a lex/env key in scope and as one element of
/// a reference path such as `dir.xxx`.
///
/// `Hash`/`Eq` are required for use as HashMap keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodePathElement {
    pub kind: NodeKind,
    pub id: NodeId,
}

impl NodePathElement {
    /// Creates an element from `(kind, id)`.
    pub fn new(kind: NodeKind, id: impl Into<NodeId>) -> Self {
        NodePathElement {
            kind,
            id: id.into(),
        }
    }
}

/// A complete reference path (a sequence of segments for a qualified path such as `dir.xxx.file.yyy`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePath(pub Vec<NodePathElement>);
