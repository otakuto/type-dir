use indexmap::IndexMap;

use crate::expr::{ExprPattern, Quant};
use crate::yaml::{EntryId, RuleName, ValueExpr, VarName};

/// One arm of a `match` (Sum elimination). Each arm unconditionally corresponds to exactly one
/// Sum constructor; the structural invariant (record-intro with a required tag) is expressed at
/// the type level rather than checked by consumers at runtime.
///
/// - `tag`: the constructor label (previously `ExprEntry.id` on the arm entry).
/// - `subtree`: the content model for this arm (previously the `Record.subtree` inside the arm).
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub tag: EntryId,
    pub subtree: Vec<ExprEntry>,
}

/// Validated entry. Invalid combinations of name matchers and subtrees are excluded at the type level.
///
/// Entries with an `id` collect one record from the named captures of the matched node
/// (for this entry) and the match name (`Record.name`). The `outputs` declaration has been removed.
#[derive(Debug, Clone)]
pub struct ExprEntry {
    pub id: Option<EntryId>,
    /// Structural path into the YAML config for this entry (e.g. `"rules.foo.rules[2]"`), used to
    /// resolve the violated entry's source span when rendering a lint error. `None` for entries
    /// synthesized at runtime (e.g. splice desugaring) or constructed directly in tests.
    pub source_path: Option<String>,
    /// Count quantifier.
    ///
    /// `Quant::Default` represents user-unspecified (Exact is required, Regex is `{1, ∞}`).
    /// `Quant::Explicit(iv)` represents the explicitly specified interval `[iv.min, iv.max]`.
    ///
    /// Attached only to dir/file entries (`ExprMatcher::Dir` / `ExprMatcher::File`). The legacy `optional` bool flag is
    /// desugared at compile time to `Quant::Explicit(Interval { min: 0, max: Some(1) })` and does
    /// not exist in the AST layer.
    pub count: Quant,
    pub matcher: ExprMatcher,
}

/// Iteration source that can be specified in the `value:` field of a `for` entry's `{id, value}`
/// map (expr layer).
#[derive(Debug, Clone)]
pub enum ExprForSource {
    /// YAML literal list.
    Literal(Vec<String>),
    /// Template string (`${id.var}` / `${var}` / bare string).
    Expr(String),
}

/// Constructor for an entry. One of: `Dir` / `File` (describes a dir/file node), `Use` (expands a
/// rule at the current position), `Group` (anonymous group / use+id record-intro), `choice`
/// (one_of/any_of), or `for` iteration.
#[derive(Debug, Clone)]
pub enum ExprMatcher {
    /// Describes a directory node (the entry side always describes the node).
    Dir {
        pattern: ExprPattern,
        subtree: ExprSubtree,
    },
    /// Describes a file node (the entry side always describes the node).
    File {
        pattern: ExprPattern,
        subtree: ExprSubtree,
    },
    /// Hermetically expands a rule at the current position (`- use: rule.X`).
    /// Inserts the rule's entries in a scope where only X's declared `with` parameters are visible.
    /// Has no name matcher (disappears after expansion).
    /// Used for content-choice backtracking when all alternatives of one_of/any_of are Use entries.
    Use {
        rule: RuleName,
        with_args: IndexMap<VarName, String>,
    },
    /// Group (anonymous group / splice+id record-intro).
    ///
    /// Non-consuming logical Map of multiplicity exactly 1. The record id lives on `ExprEntry.id`.
    /// For enforcement the subtree is expanded transparently in the same scope; for collection one
    /// wrapping record is produced whose children are the subtree's collected id sets.
    ///
    /// This variant unifies two surface forms:
    /// - `- id: Y / rules: [...]`  (anonymous group with id)
    /// - `- use: rule.X / id: Y`    (splice+id, desugared to a Group wrapping a bare Splice)
    Group { subtree: Vec<ExprEntry> },
    /// The number of realized alternatives of a choice (one_of/any_of) falls within [min, max]
    /// (max=None means unbounded).
    Choice {
        min: usize,
        max: Option<usize>,
        body: Vec<ExprEntry>,
    },
    /// `for` loop. Scalar-binds each value of `source` to the bound variable `var` and expands
    /// `body` into the content model of the current node. Has no name matcher.
    For {
        var: VarName,
        source: ExprForSource,
        body: Vec<ExprEntry>,
    },
    /// Sum elimination. Dispatches on the scrutinee record's `tag` and transparently expands the
    /// matching arm's subtree into the current node's content model (non-consuming, multiplicity 1).
    ///
    /// - `scrutinee`: inner key of `${...}` (e.g. `"c"`); must be a for-bound single Record var.
    /// - `arms`: each arm carries its constructor `tag` and the associated content-model `subtree`.
    ///
    /// Enforcement dispatch and exhaustiveness checking are implemented in Bundle B.
    Match {
        scrutinee: String,
        arms: Vec<MatchArm>,
    },
    /// Non-consuming observation entry. Scans the current level's children against each
    /// `alternatives` pattern and accumulates matching records into `sets[id]`.
    ///
    /// - All alternatives share the same named capture set (statically enforced at load time).
    /// - Matched children are NOT consumed: a separate enforcing entry (e.g., a `for` loop) must
    ///   also declare them, otherwise they will be reported as E001 (undeclared).
    /// - `id` lives on the enclosing `ExprEntry.id`; `body` are the dir/file alternative patterns.
    Fetch { body: Vec<ExprEntry> },
    /// `value` variable binding (`- id: x / value: ...`). Non-consuming, multiplicity exactly 1.
    /// Evaluates `value` (interpolating each template via `resolve_template` against the current
    /// scope) and binds the result to `var` in the `value` namespace (`${value.<var>}`). Bindings
    /// are sequential-let: the binding is visible to subsequent sibling entries, their children,
    /// and `with:` arguments in the same `rules` block. Has no name matcher and produces no record.
    Value { var: VarName, value: ValueExpr },
}

/// Subtree of an entry that owns a matcher.
#[derive(Debug, Clone)]
pub enum ExprSubtree {
    /// Leaf (no recursion).
    Leaf,
    /// Recurses via inline entries (`- use: rule.X` is written as a Splice entry within inline).
    Inline(Vec<ExprEntry>),
}
