mod repr;

#[cfg(test)]
#[path = "yaml_entry_tests/tests.rs"]
mod tests;

use crate::yaml::{EntryId, RuleName, ValueExpr, VarName};
use indexmap::IndexMap;
use serde::Deserialize;

use super::YamlPattern;
use repr::YamlEntryRepr;

/// Iteration source specified in the `value:` field of a `for` entry's `{id, value}` map.
///
/// With untagged deserialization, a YAML list (`["a","b"]`) is interpreted as `Literal` and
/// a string (`${value.var}` / bare string) is interpreted as `Expr`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum YamlForSource {
    /// YAML literal list (`["a", "b"]` format).
    Literal(Vec<String>),
    /// Template string (`${id.var}` / `${var}` / bare string).
    Expr(String),
}

/// Kind-specific data for a `YamlEntry`.
///
/// Horizontal fields shared across multiple kinds (`id`, `optional`, `min`, `max`, `count`)
/// are kept on `YamlEntry` directly; kind-exclusive data lives here.
#[derive(Debug, Clone)]
pub enum YamlEntryKind {
    /// `dir:` entry. Describes its own filesystem directory node.
    Dir {
        pattern: YamlPattern,
        /// Inline sub-entries (`::` body), if any.
        body: Option<Vec<YamlEntry>>,
        /// Illegally colocated `use:` value (present ⟹ `DirFileWithRule` error).
        colocated_use_ref: Option<RuleName>,
    },
    /// `file:` entry. Describes its own filesystem file node.
    File {
        pattern: YamlPattern,
        /// Inline sub-entries (`::` body), if any.
        body: Option<Vec<YamlEntry>>,
        /// Illegally colocated `use:` value (present ⟹ `DirFileWithRule` error).
        colocated_use_ref: Option<RuleName>,
    },
    /// Rule invocation (`- use: rule.X`), optionally with `with:`.
    /// When `entry.id` is also set, the splice is desugared to a Record at compile time.
    /// When `colocated_rules` is `Some`, it carries illegally colocated `rules:` sub-entries
    /// that were found alongside `use:` in YAML; `check_entry_combination` rejects this with
    /// `SpliceWithSubtree`.
    Use {
        rule: RuleName,
        /// Values passed to the referenced rule (name → value).
        with_args: IndexMap<VarName, String>,
        /// Illegally colocated `rules:` sub-entries (present ⟹ `SpliceWithSubtree` error).
        colocated_rules: Option<Vec<YamlEntry>>,
    },
    /// Group (record-intro): declared with the explicit `group:` marker keyword (no dir/file/use).
    /// `entry.id` carries the record id (id-bearing ⟹ record-intro, referenceable as
    /// `${group.<id>}`; id-less ⟹ transparent group).
    ///
    /// `explicit_marker` records whether the `group:` keyword was written in the YAML. The implicit
    /// form (`rules:`/`::` with no dir/file/use and no `group:`) is no longer valid and is produced
    /// with `explicit_marker == false` so `check_entry_combination` can reject it with a message
    /// directing the author to add `group:`.
    Group {
        body: Vec<YamlEntry>,
        explicit_marker: bool,
    },
    /// Group / cardinality-bounded selection (`one_of` / `any_of` / `choice`).
    ///
    /// `one_of` maps to (min=1, max=Some(1)), `any_of` to (min=1, max=None),
    /// `choice` carries user-specified min/max directly.
    Choice {
        min: usize,
        max: Option<usize>,
        body: Vec<YamlEntry>,
    },
    /// `for` loop entry.
    ///
    /// When `entry.id` is set, each binding's collected body is wrapped in one `Record` and
    /// accumulated as a `RecordList` under `out[id]`. Without an `id`, body ids are collected
    /// transparently into the outer output (current behaviour).
    For {
        var: VarName,
        source: YamlForSource,
        body: Vec<YamlEntry>,
    },
    /// `match` Sum elimination entry.
    Match {
        scrutinee: String,
        body: Vec<YamlEntry>,
    },
    /// `fetch` non-consuming observation entry. `entry.id` carries the fetch id.
    Fetch { body: Vec<YamlEntry> },
    /// `value` variable binding entry (`- id: x / value: ...`).
    ///
    /// `var` is the bound variable name (the entry's `id:` key). It is NOT a capture, so
    /// `YamlEntry.id` is set to `None` to keep the binding off the Γ_set record-id path
    /// (it lives in the `value` namespace instead). `value` is the scalar/list expression.
    Value { var: VarName, value: ValueExpr },
}

/// Represents one item in roots / entries.
///
/// Count constraints are specified by four sibling keys: `optional` / `min` / `max` / `count`
/// (scalar only). Map form for `count:` is not supported.
/// This struct is constructed via `YamlEntryRepr`.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "YamlEntryRepr")]
pub struct YamlEntry {
    /// Entry id (dir/file/splice+id/anon-group/fetch/Choice with id).
    pub id: Option<EntryId>,
    /// `optional: true` flag. Sets the effective min to 0 (`optional: false` is equivalent to absence).
    pub optional: Option<bool>,
    /// Count lower bound (`min: n`).
    pub min: Option<usize>,
    /// Count upper bound (`max: n`).
    pub max: Option<usize>,
    /// Exact count (`count: n`, scalar only). The map form `count: {min, max}` has been removed.
    pub count: Option<usize>,
    /// Kind-specific data.
    pub kind: YamlEntryKind,
}

impl YamlEntry {
    /// Creates a `YamlEntry` with only `id` and `kind` set; all optional count
    /// fields are left `None`.
    fn with_kind(id: Option<EntryId>, kind: YamlEntryKind) -> YamlEntry {
        YamlEntry {
            id,
            optional: None,
            min: None,
            max: None,
            count: None,
            kind,
        }
    }
}

impl From<YamlEntryRepr> for YamlEntry {
    fn from(repr: YamlEntryRepr) -> YamlEntry {
        match repr {
            YamlEntryRepr::OneOf { id, body } => YamlEntry::with_kind(
                id,
                YamlEntryKind::Choice {
                    min: 1,
                    max: Some(1),
                    body,
                },
            ),
            YamlEntryRepr::AnyOf { id, body } => YamlEntry::with_kind(
                id,
                YamlEntryKind::Choice {
                    min: 1,
                    max: None,
                    body,
                },
            ),
            YamlEntryRepr::Choice { id, min, max, body } => {
                YamlEntry::with_kind(id, YamlEntryKind::Choice { min, max, body })
            }
            YamlEntryRepr::For {
                var,
                source,
                id,
                body,
            } => YamlEntry::with_kind(id, YamlEntryKind::For { var, source, body }),
            YamlEntryRepr::Match { scrutinee, body } => {
                YamlEntry::with_kind(None, YamlEntryKind::Match { scrutinee, body })
            }
            YamlEntryRepr::Fetch { id, body } => {
                YamlEntry::with_kind(Some(id), YamlEntryKind::Fetch { body })
            }
            YamlEntryRepr::Value { var, value } => YamlEntry::with_kind(
                // The binding name lives on the kind (`var`), not on `id`: a value binding is not a
                // capture, so keeping `id` None avoids registering it on the Γ_set record-id path.
                None,
                YamlEntryKind::Value { var, value },
            ),
            YamlEntryRepr::Plain(p) => {
                let p = *p;
                // Determine kind from the available fields.
                let kind = if let Some(dir_pat) = p.dir {
                    YamlEntryKind::Dir {
                        pattern: dir_pat,
                        body: p.body,
                        // Carry colocated `use:` through so check_entry_combination can reject it.
                        colocated_use_ref: p.use_ref.map(|u| u.0),
                    }
                } else if let Some(file_pat) = p.file {
                    YamlEntryKind::File {
                        pattern: file_pat,
                        body: p.body,
                        // Carry colocated `use:` through so check_entry_combination can reject it.
                        colocated_use_ref: p.use_ref.map(|u| u.0),
                    }
                } else if let Some(use_ref) = p.use_ref {
                    YamlEntryKind::Use {
                        rule: use_ref.0,
                        with_args: p.with_args,
                        // Carry colocated `rules:` through so check_entry_combination can reject it.
                        colocated_rules: p.body,
                    }
                } else if p.group {
                    // Explicit `group:` marker: a record-intro group. Contents come from `::`.
                    YamlEntryKind::Group {
                        body: p.body.unwrap_or_default(),
                        explicit_marker: true,
                    }
                } else if let Some(inline_body) = p.body {
                    // Implicit anonymous group (`rules:`/`::` with no dir/file/use and no `group:`).
                    // This form is no longer valid; carry the rules through with
                    // `explicit_marker == false` so check_entry_combination rejects it (directing the
                    // author to add `group:`).
                    YamlEntryKind::Group {
                        body: inline_body,
                        explicit_marker: false,
                    }
                } else {
                    // No matcher at all: produces an "empty" dir/file-like placeholder.
                    // check_entry_combination will reject this before compile.
                    // Use a sentinel so the kind field is always populated.
                    YamlEntryKind::Group {
                        body: vec![],
                        explicit_marker: false,
                    }
                };
                YamlEntry {
                    id: p.id,
                    optional: p.optional,
                    min: p.min,
                    max: p.max,
                    count: p.count,
                    kind,
                }
            }
        }
    }
}
