use std::collections::HashSet;

use indexmap::IndexMap;

use crate::yaml::RuleName;

/// Whether an id-bearing entry matches a directory or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Dir,
    File,
}

impl NodeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Dir => "dir",
            NodeKind::File => "file",
        }
    }
}

/// A reference to a child id, either an inline sub-shape or a lazy reference to a named rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildRef {
    /// The child id has an inline-derived shape (from an id-bearing Own dir/file entry).
    Inline(IdShape),
    /// The child id was introduced by a splice+id entry; the shape is the public id of rule `name`.
    RuleRef(RuleName),
}

/// Static shape of an id: the node kind (dir or file), the set of capture names (from the id
/// entry's own pattern), and the map of child id names to their shape references.
///
/// `kind` records whether the id-bearing entry is a directory or a file.
/// `captures` holds named capture names from the id entry's pattern (`name` is reserved and excluded).
/// `child_ids` maps child set field names to their `ChildRef`, built by the (A') transparency rule.
/// `sum_alts` is `Some` only when this shape was derived from an id-bearing Group (Sum occurrence
/// collector). It maps each alternative id (tag) to the per-alternative `IdShape`, enabling
/// per-arm narrowing in `match` expressions: within a `- id: tag / rules: [...]` arm the
/// scrutinee's captures are restricted to those declared by that alternative only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdShape {
    pub kind: NodeKind,
    pub captures: HashSet<String>,
    pub child_ids: IndexMap<String, ChildRef>,
    /// Per-alternative shapes for Sum ids (id-bearing Group entries).
    /// `None` for non-Sum ids (plain dir/file id entries, splice+id entries).
    pub sum_alts: Option<IndexMap<String, IdShape>>,
}
