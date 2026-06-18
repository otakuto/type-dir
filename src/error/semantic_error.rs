use thiserror::Error;

/// A rule-definition error found while validating the rules of a `.dir-lint.yaml` (the `compile`
/// pass). These mean "the rules themselves don't make sense", independent of any target directory.
///
/// The `context_path()` method returns a structural path string (e.g. `"rules.foo.rules[2]"`) usable
/// as a key into a `SpanIndex`; `None` when no config-structure position applies.
#[derive(Debug, Clone, Error)]
pub enum SemanticError {
    /// A referenced rule name is not defined.
    #[error("undefined rule referenced: `{name}` (context: {context})")]
    UndefinedRule { name: String, context: String },

    /// An id is duplicated.
    #[error("duplicate id: `{id}`")]
    DuplicateId { id: String },

    /// A rule name is defined more than once in the top-level `rules:` list.
    #[error("duplicate rule definition: `{rule}`")]
    DuplicateRule { rule: String },

    /// A cycle formed solely by splice edges without passing through any dir/file node.
    #[error(
        "infinite splice: rule expansion cycles without passing through any dir/file node: {}",
        cycle.join(" -> ")
    )]
    InfiniteSplice { cycle: Vec<String> },

    /// A dir/file entry has a rule placed alongside it.
    #[error(
        "rule cannot be placed alongside a dir/file entry (nest it under `::` as `- use: rule.X`): {context}"
    )]
    DirFileWithRule { context: String },

    /// A bare rule (splice) entry has rules/id placed alongside it (splice is expansion only).
    #[error(
        "rules cannot be placed alongside a bare rule (splice) entry (splice is expansion only; use `id:` for record-intro, or an anonymous group `- id: x` with `::`): {context}"
    )]
    SpliceWithSubtree { context: String },

    /// A key not declared in `with` was specified.
    #[error("undeclared with `{with}` passed to rule `{rule}`: {context}")]
    UnknownWith {
        rule: String,
        with: String,
        context: String,
    },

    /// A rule references a variable outside its declarations (only with params/captures are allowed).
    #[error(
        "rule `{rule}` references undeclared variable `${{{reference}}}` (only with params or own captures are allowed)"
    )]
    RuleUndeclaredRef { rule: String, reference: String },

    /// `${rule.<id>}` used in a value position; only valid in type positions (with: declarations).
    /// Use `${use.<id>}` to reference a splice instance value.
    #[error(
        "`${{rule.{rule_id}}}` is a type reference and cannot be used in a value position (rule: {rule}); use `${{use.{rule_id}}}` to reference the splice instance value"
    )]
    RuleNsInValuePosition { rule: String, rule_id: String },

    /// A dotted reference is not kind-qualified (e.g. `${x.stem}` instead of `${x.regex.stem}`).
    #[error(
        "unqualified reference `${{{reference}}}` (context: {context}); use `.regex.<group>` (`.regex.0` for full match), `.dir.<id>`, or `.file.<id>`"
    )]
    UnqualifiedReference { reference: String, context: String },

    /// The caller's with argument does not conform to the declared shape of the rule.
    #[error("with `{with}` does not match the declared shape of rule `{rule}`: {detail}")]
    WithShapeMismatch {
        rule: String,
        with: String,
        detail: String,
    },

    /// A with param declared as `RuleType` references a rule name that is not defined.
    #[error("with `{with}` of rule `{rule}` references undefined rule `{ref_rule}` as its type")]
    UndefinedShapeRule {
        rule: String,
        with: String,
        ref_rule: String,
    },

    /// A qualified `.dir.`/`.file.` reference is used for a child id whose actual kind does not match.
    #[error("reference `{reference}` uses `.{expected}.` but the id is a {actual} (rule: {rule})")]
    NodeKindMismatch {
        rule: String,
        reference: String,
        expected: String,
        actual: String,
    },

    /// A `match` (Sum elimination) does not cover its scrutinee's Sum tags exactly.
    #[error(
        "non-exhaustive match on `${{{scrutinee}}}` (rule: {rule}):{}",
        arms_detail(missing, extra)
    )]
    NonExhaustiveMatch {
        rule: String,
        scrutinee: String,
        missing: Vec<String>,
        extra: Vec<String>,
    },

    /// A `match` scrutinee is not bound to a Sum (id-bearing Group).
    #[error(
        "match scrutinee `${{{scrutinee}}}` is not a Sum (it must iterate an id-bearing one_of/any_of/choice) (rule: {rule})"
    )]
    MatchOnNonSum { rule: String, scrutinee: String },

    /// A dir/file entry has one or more named captures (`(?<name>...)`) but no `id`.
    #[error(
        "named capture(s) [{}] on entry without `id` (rule: `{rule}`, {context}): add `id:` to make captures accessible as record fields",
        captures.join(", ")
    )]
    CaptureWithoutId {
        rule: String,
        context: String,
        captures: Vec<String>,
    },

    /// The named capture sets of a `fetch` entry's alternatives do not all agree.
    #[error(
        "fetch `{fetch_id}` alt[{alt_index}] has capture set [{}] but expected [{}] (rule: `{rule}`): all fetch alternatives must declare the same named captures",
        actual.join(", "),
        expected.join(", ")
    )]
    FetchCaptureMismatch {
        rule: String,
        fetch_id: String,
        alt_index: usize,
        expected: Vec<String>,
        actual: Vec<String>,
    },

    /// An invalid pattern specification (e.g. malformed regex).
    #[error("invalid pattern: {reason} ({context})")]
    InvalidPattern { context: String, reason: String },

    /// An unsupported config version was specified (only 0 is supported).
    #[error("unsupported config version: {version} (only 0 is supported)")]
    UnsupportedVersion { version: u32 },

    /// A `${id}` reference resolves to an id declared later in source order (forward reference).
    #[error("forward reference to id `{id}` (declared later) in rule `{rule}`: `{reference}`")]
    ForwardReference {
        rule: String,
        reference: String,
        id: String,
    },

    /// A bare (un-namespaced) `${...}` reference was used. References must name a namespace head
    /// (`dir.`/`file.`/`group.`/`choice.`/`use.`/`for.`/`fetch.`/`value.`/`with.`/`rule.`).
    #[error(
        "bare reference `${{{reference}}}` in rule `{rule}`; references must be namespaced (e.g. `${{dir.<id>}}`, `${{file.<id>}}`, `${{value.<var>}}`)"
    )]
    BareReference { rule: String, reference: String },

    /// An entry with a `::` (`rules:`) block but no dir/file/use and no explicit `group:` marker.
    /// Record-intro groups must be declared explicitly with the `group:` keyword.
    #[error(
        "record-intro group must be declared with the explicit `group:` marker (an entry with `::` but no dir/file/use): {context}"
    )]
    ImplicitGroup { context: String },
}

impl SemanticError {
    /// Returns the diagnostic code (`SM001`..).
    pub fn code(&self) -> &'static str {
        match self {
            SemanticError::UndefinedRule { .. } => "SM001",
            SemanticError::DuplicateId { .. } => "SM002",
            SemanticError::DuplicateRule { .. } => "SM019",
            SemanticError::InfiniteSplice { .. } => "SM004",
            SemanticError::DirFileWithRule { .. } => "SM005",
            SemanticError::SpliceWithSubtree { .. } => "SM006",
            SemanticError::UnknownWith { .. } => "SM007",
            SemanticError::RuleUndeclaredRef { .. } => "SM008",
            SemanticError::UnqualifiedReference { .. } => "SM009",
            SemanticError::WithShapeMismatch { .. } => "SM010",
            SemanticError::UndefinedShapeRule { .. } => "SM011",
            SemanticError::NodeKindMismatch { .. } => "SM012",
            SemanticError::NonExhaustiveMatch { .. } => "SM013",
            SemanticError::MatchOnNonSum { .. } => "SM014",
            SemanticError::CaptureWithoutId { .. } => "SM015",
            SemanticError::FetchCaptureMismatch { .. } => "SM016",
            SemanticError::InvalidPattern { .. } => "SM017",
            SemanticError::UnsupportedVersion { .. } => "SM018",
            SemanticError::RuleNsInValuePosition { .. } => "SM020",
            SemanticError::ForwardReference { .. } => "SM021",
            SemanticError::BareReference { .. } => "SM022",
            SemanticError::ImplicitGroup { .. } => "SM023",
        }
    }

    /// Returns the structural path into the YAML config most closely associated with this error.
    pub fn context_path(&self) -> Option<String> {
        match self {
            // Variants carrying a full `rules.<rule>.rules[<i>]...` context path — return as-is.
            SemanticError::DirFileWithRule { context }
            | SemanticError::SpliceWithSubtree { context }
            | SemanticError::ImplicitGroup { context }
            | SemanticError::InvalidPattern { context, .. }
            | SemanticError::UndefinedRule { context, .. }
            | SemanticError::UnknownWith { context, .. } => non_empty(context),

            // UnqualifiedReference.context is the rule name (not a full path).
            SemanticError::UnqualifiedReference { context, .. } => rule_path(context),

            // Variants that only carry a rule name — point to the rule definition.
            SemanticError::RuleUndeclaredRef { rule, .. }
            | SemanticError::RuleNsInValuePosition { rule, .. }
            | SemanticError::ForwardReference { rule, .. }
            | SemanticError::WithShapeMismatch { rule, .. }
            | SemanticError::UndefinedShapeRule { rule, .. }
            | SemanticError::NodeKindMismatch { rule, .. }
            | SemanticError::NonExhaustiveMatch { rule, .. }
            | SemanticError::MatchOnNonSum { rule, .. }
            | SemanticError::CaptureWithoutId { rule, .. }
            | SemanticError::BareReference { rule, .. }
            | SemanticError::FetchCaptureMismatch { rule, .. } => rule_path(rule),

            // Variants with no structural config position.
            SemanticError::DuplicateId { .. }
            | SemanticError::DuplicateRule { .. }
            | SemanticError::InfiniteSplice { .. }
            | SemanticError::UnsupportedVersion { .. } => None,
        }
    }

    /// Returns a supplemental fix hint shown beneath the diagnostic, if any.
    pub fn fix_hint(&self) -> Option<&'static str> {
        match self {
            SemanticError::InfiniteSplice { .. } => Some(
                "break the cycle by passing through a dir/file entry to consume a node, or remove the circular reference",
            ),
            SemanticError::UnqualifiedReference { .. } => Some(
                "kind-qualify the reference: use `.regex.<group>` (`.regex.0` for full match), `.dir.<id>`, or `.file.<id>`",
            ),
            SemanticError::NonExhaustiveMatch { .. } => Some(
                "add one arm (`- id: <tag>` then `::`) per alternative id of the scrutinee's id-bearing group, and remove arms whose id is not an alternative",
            ),
            SemanticError::MatchOnNonSum { .. } => Some(
                "the scrutinee must be a `for` variable iterating an id-bearing one_of/any_of/choice (`id:` on the group); its alternative ids become the match tags",
            ),
            SemanticError::CaptureWithoutId { .. } => Some(
                "add `id: <name>` to the entry so that named captures become accessible as `${<name>.regex.<capture>}`, or remove the named captures if they are not needed",
            ),
            SemanticError::FetchCaptureMismatch { .. } => Some(
                "ensure every `fetch` alternative declares the same named capture names so that `${fetch_id.regex.<name>}` is well-defined for all records",
            ),
            _ => None,
        }
    }
}

/// Returns `Some(s)` when non-empty, else `None`.
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

/// Builds the structural config path for a rule name (e.g. `rules.foo`).
/// Returns `None` when `name` is empty.
fn rule_path(name: &str) -> Option<String> {
    if name.is_empty() {
        None
    } else {
        Some(format!("rules.{name}"))
    }
}

/// Renders the missing/dead-arm detail suffix of a non-exhaustive match.
fn arms_detail(missing: &[String], extra: &[String]) -> String {
    let mut detail = String::new();
    if !missing.is_empty() {
        detail.push_str(&format!(" missing arms: [{}]", missing.join(", ")));
    }
    if !extra.is_empty() {
        detail.push_str(&format!(" dead arms: [{}]", extra.join(", ")));
    }
    detail
}
