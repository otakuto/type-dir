use crate::yaml::{EntryId, RuleName, VarName};
use indexmap::IndexMap;

/// Declaration shape for one `with` parameter of a rule (the `type:` field).
///
/// The explicit type system replaces the former ambiguous `null` (scalar) / `[]` (array)
/// encodings. A declaration is one of:
///
/// - Primitives: `type.string` / `type.number` / `type.bool` → `String` / `Number` / `Bool`.
/// - Array: `[T]` (a one-element YAML sequence whose element is itself a type) → `Array(T)`.
///   Nested arrays such as `[[type.number]]` describe a 2-dimensional array.
/// - Object: `{a: type.string, b: [[type.number]], ...}` (a YAML mapping whose values are types)
///   → `Object(fields)`, order-preserving.
/// - `RuleType { rule, path }`: non-terminal type reference (in YAML: dotted string `rule.<rule>` or
///   `rule.<rule>.<kind>.<name>...` with kind ∈ dir/file/regex, mirroring the reference grammar
///   `rule.R.dir.node`). The parameter expects a value whose static shape is derived from rule `rule`.
///   When `path` is empty the shape is the single public id of `rule`; when `path` is non-empty each
///   element (a `<name>` taken from a `<kind>.<name>` pair) selects a named child id, drilling into the
///   sub-tree of that rule's shape. The kinds are required syntactically but not used for resolution.
/// - `Scalar`: an internal permissive-scalar fallback. It is never produced by the parser (the
///   grammar requires an explicit primitive); it only arises as the lenient default when an invalid
///   declaration is force-compiled after `check_with_shapes` has already reported the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WithShape {
    Scalar,
    String,
    Number,
    Bool,
    Array(Box<WithShape>),
    Object(IndexMap<VarName, WithShape>),
    RuleType { rule: RuleName, path: Vec<EntryId> },
}
