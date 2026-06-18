/// The head of a parsed `${...}` reference, describing the namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefHead {
    /// `rule.<rule_id>.<hops>` — reference into a splice record set identified by `rule_id`.
    /// Tail hops navigate into that record set (e.g. `rule.R.dir.x`, `rule.R.with.p.regex.stem`).
    /// Valid only in **type positions** (with: declarations, `- use: rule.X` invoke targets).
    /// Using `${rule.X}` in a value position is a semantic error; use `${use.<id>}` instead.
    RuleNs { rule_id: String, tail: Vec<Hop> },
    /// `with.<param>.<hops>` — reference into a with-param of the current rule.
    /// Tail hops navigate further (e.g. `with.feature_dir.regex.stem`).
    WithNs { param: String, tail: Vec<Hop> },
    /// `value.<var>.<hops>` — reference into a `value:` binding of the current rule.
    /// `value` bindings are scalars or string lists, so the tail is usually empty
    /// (or used as a `for` source for list bindings). Tail hops navigate further when present.
    ValueNs { var: String, tail: Vec<Hop> },
    /// `use.<id>.<hops>` — value reference into a splice instance bound by `- use: rule.X / id: <id>`.
    /// Resolves to the public output of that splice instance, with auto-unwrap when the instance
    /// has a single public id (the wrapper Record's single child set is returned directly).
    /// Tail hops navigate further into the resolved record set.
    UseNs { id: String, tail: Vec<Hop> },
    /// `for.<id>.<hops>` — value reference into the per-binding wrapped records accumulated by a
    /// `for` entry that carries `id: <id>`. Each iteration's collected body is wrapped in one
    /// `Record` and pushed to `out[id]` as a `RecordList`. Tail hops navigate further (e.g.
    /// `for.loop1.dir.node`, `for.loop1.file.cfg`, `for.loop1.regex.stem`).
    ForNs { id: String, tail: Vec<Hop> },
    /// `fetch.<id>.<hops>` — value reference into the records collected by a `fetch` entry that
    /// carries `id: <id>`. A `fetch` entry observes (without consuming) the children matching its
    /// alt patterns and binds the resulting record set to `out[id]` (`Γ_set`). Tail hops navigate
    /// further (e.g. `fetch.dirs.regex.n`). Resolved on the same path as [`RefHead::ForNs`].
    FetchNs { id: String, tail: Vec<Hop> },
    /// `dir.<id>.<hops>` — kind-qualified reference into a self-owned dir entry identified by `id`.
    /// Tail hops navigate into that record set (e.g. `dir.x.file.y`, `dir.x.regex.stem`).
    /// Coexists with the bare `${id}` form and resolves on the same path.
    DirNs { id: String, tail: Vec<Hop> },
    /// `file.<id>.<hops>` — kind-qualified reference into a self-owned file entry identified by `id`.
    /// Tail hops navigate further (e.g. `file.cfg.regex.stem`). Resolved like [`RefHead::DirNs`].
    FileNs { id: String, tail: Vec<Hop> },
    /// `group.<id>.<hops>` — kind-qualified reference into a self-owned group entry identified by `id`.
    /// Tail hops navigate further. Resolved like [`RefHead::DirNs`].
    GroupNs { id: String, tail: Vec<Hop> },
    /// `choice.<id>.<hops>` — kind-qualified reference into a self-owned choice entry identified by `id`.
    /// Tail hops navigate further. Resolved like [`RefHead::DirNs`].
    ChoiceNs { id: String, tail: Vec<Hop> },
    /// Bare (un-namespaced) head — a reference whose first segment is not one of the recognized
    /// namespace keywords. Retained for **diagnostics only**: bare references are rejected at
    /// compile time (`SemanticError::BareReference`). The string is the raw head segment and any
    /// dotted tail is parsed into `Ref.hops` so the diagnostic can describe the whole reference.
    Bare(String),
}

/// A parsed `${...}` reference: a head (namespace + entry) plus optional navigation hops.
///
/// When the head is `RefHead::Bare` the hops carry the dotted tail (kept only for diagnostics).
/// When the head is a namespace variant (`RuleNs`/`WithNs`/`DirNs`/...) the tail hops are embedded
/// in the head variant itself; `hops` on the `Ref` is always empty for those heads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub head: RefHead,
    /// Navigation hops from the head binding (only non-empty when `head == RefHead::Bare`).
    pub hops: Vec<Hop>,
}

/// One navigation or projection step in a reference path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hop {
    /// `.regex.<g>` — a regex capture group (scalar leaf).
    /// Use `"0"` for the full match, `"1"`, `"2"`, ... for positional groups,
    /// or a name for named groups.
    Regex(String),
    /// `.dir.<id>` — a child record set of kind dir.
    Dir(String),
    /// `.file.<id>` — a child record set of kind file.
    File(String),
    /// `.choice.<id>` — a child record set produced by a `one_of`/`any_of`/`choice` with that id.
    Choice(String),
    /// `.group.<id>` — a child record set produced by a record-intro group with that id.
    Group(String),
    /// `.for.<id>` — a child record set produced by a `for` entry carrying that id.
    For(String),
    /// `.fetch.<id>` — a child record set produced by a `fetch` entry carrying that id.
    Fetch(String),
    /// Unqualified single field `.foo`. This variant is kept for internal recognition so that
    /// `check_rule_var_scope` can reject it with `UnqualifiedReference` (E023). New code must
    /// never produce `Hop::Field` references; only already-invalid inputs parse into it
    /// (when `head == RefHead::Bare` and the dot-segment is not a recognized keyword).
    Field(String),
}

/// Parse the inner key of a `${...}` reference (the `${`/`}` delimiters must be
/// stripped by the caller before calling this function).
///
/// Namespace dispatch on the first segment:
/// - `"rule"` → [`RefHead::RuleNs`]: second segment is the rule id; remaining segments are parsed
///   as hops, with the special handling that `"with"` at any hop position consumes the next
///   segment as the param name (producing no further `Hop` type — the tail carries those hops).
///   Full tail is stored in `RefHead::RuleNs.tail`; `Ref.hops` is empty.
/// - `"with"` → [`RefHead::WithNs`]: second segment is the param name; remaining segments are
///   parsed as hops. Full tail stored in `RefHead::WithNs.tail`; `Ref.hops` is empty.
/// - `"value"` → [`RefHead::ValueNs`]: second segment is the `value:` binding name; remaining
///   segments are parsed as hops. Full tail stored in `RefHead::ValueNs.tail`; `Ref.hops` is empty.
/// - `"use"` → [`RefHead::UseNs`]: second segment is the splice instance id; remaining segments
///   are parsed as hops. Full tail stored in `RefHead::UseNs.tail`; `Ref.hops` is empty.
/// - `"for"` → [`RefHead::ForNs`]: second segment is the for-entry id; remaining segments are
///   parsed as hops. Full tail stored in `RefHead::ForNs.tail`; `Ref.hops` is empty.
/// - `"fetch"` → [`RefHead::FetchNs`]: second segment is the fetch-entry id; remaining segments are
///   parsed as hops. Full tail stored in `RefHead::FetchNs.tail`; `Ref.hops` is empty.
/// - `"dir"` → [`RefHead::DirNs`]: second segment is the dir entry id; remaining segments are parsed
///   as hops. Full tail stored in `RefHead::DirNs.tail`; `Ref.hops` is empty.
/// - `"file"` → [`RefHead::FileNs`]: second segment is the file entry id; remaining segments are
///   parsed as hops. Full tail stored in `RefHead::FileNs.tail`; `Ref.hops` is empty.
/// - `"group"` → [`RefHead::GroupNs`]: second segment is the group entry id; remaining segments are
///   parsed as hops. Full tail stored in `RefHead::GroupNs.tail`; `Ref.hops` is empty.
/// - `"choice"` → [`RefHead::ChoiceNs`]: second segment is the choice entry id; remaining segments are
///   parsed as hops. Full tail stored in `RefHead::ChoiceNs.tail`; `Ref.hops` is empty.
/// - anything else → [`RefHead::Bare`]: a non-namespaced head, kept for diagnostics only.
///   Remaining segments are parsed as hops stored in `Ref.hops` using the standard keyword
///   dispatch (regex/dir/file/other→Field). The compile pass rejects bare references.
pub fn parse_ref(key: &str) -> Ref {
    let segments: Vec<&str> = key.split('.').collect();

    if segments.is_empty() {
        return Ref {
            head: RefHead::Bare(String::new()),
            hops: vec![],
        };
    }

    let first = segments[0];

    match first {
        "rule" => {
            // rule.<rule_id>.<hops...>
            let (rule_id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::RuleNs { rule_id, tail },
                hops: vec![],
            }
        }
        "with" => {
            // with.<param>.<hops...>
            let (param, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::WithNs { param, tail },
                hops: vec![],
            }
        }
        "value" => {
            // value.<var>.<hops...>
            let (var, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::ValueNs { var, tail },
                hops: vec![],
            }
        }
        "use" => {
            // use.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::UseNs { id, tail },
                hops: vec![],
            }
        }
        "for" => {
            // for.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::ForNs { id, tail },
                hops: vec![],
            }
        }
        "fetch" => {
            // fetch.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::FetchNs { id, tail },
                hops: vec![],
            }
        }
        "dir" => {
            // dir.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::DirNs { id, tail },
                hops: vec![],
            }
        }
        "file" => {
            // file.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::FileNs { id, tail },
                hops: vec![],
            }
        }
        "group" => {
            // group.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::GroupNs { id, tail },
                hops: vec![],
            }
        }
        "choice" => {
            // choice.<id>.<hops...>
            let (id, tail) = extract_id_and_tail(&segments);
            Ref {
                head: RefHead::ChoiceNs { id, tail },
                hops: vec![],
            }
        }
        head => {
            // Bare (non-namespaced) head: parse remaining as hops (diagnostics only).
            let hops = parse_hops(&segments[1..]);
            Ref {
                head: RefHead::Bare(head.to_string()),
                hops,
            }
        }
    }
}

/// Extracts the id (second segment) and tail hops from a segments slice.
///
/// Used by namespace arms in `parse_ref` that all follow the same `<ns>.<id>.<hops...>` shape.
fn extract_id_and_tail(segments: &[&str]) -> (String, Vec<Hop>) {
    let id = segments.get(1).copied().unwrap_or("").to_string();
    let len = segments.len();
    let tail = parse_hops(&segments[2.min(len)..]);
    (id, tail)
}

/// Parses a slice of dot-split segments into a `Vec<Hop>`.
///
/// Keyword dispatch (each producer keyword consumes the next segment as the id, defaulting to `""`):
/// - `"regex"`  → `Hop::Regex`  (group name).
/// - `"dir"`    → `Hop::Dir`    (dir child id).
/// - `"file"`   → `Hop::File`   (file child id).
/// - `"choice"` → `Hop::Choice` (one_of/any_of/choice child id).
/// - `"group"`  → `Hop::Group`  (record-intro group child id).
/// - `"for"`    → `Hop::For`    (for-entry child id).
/// - `"fetch"`  → `Hop::Fetch`  (fetch-entry child id).
/// - anything else → `Hop::Field` (legacy/invalid; consumes 1 segment).
fn parse_hops(segments: &[&str]) -> Vec<Hop> {
    let mut hops = Vec::new();
    let mut idx = 0;
    while idx < segments.len() {
        let seg = segments[idx];
        match seg {
            "regex" => {
                let group = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::Regex(group));
                idx += 2;
            }
            "dir" => {
                let id = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::Dir(id));
                idx += 2;
            }
            "file" => {
                let id = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::File(id));
                idx += 2;
            }
            "choice" => {
                let id = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::Choice(id));
                idx += 2;
            }
            "group" => {
                let id = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::Group(id));
                idx += 2;
            }
            "for" => {
                let id = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::For(id));
                idx += 2;
            }
            "fetch" => {
                let id = segments.get(idx + 1).copied().unwrap_or("").to_string();
                hops.push(Hop::Fetch(id));
                idx += 2;
            }
            other => {
                hops.push(Hop::Field(other.to_string()));
                idx += 1;
            }
        }
    }
    hops
}

#[cfg(test)]
#[path = "ref_path_tests/tests.rs"]
mod tests;
