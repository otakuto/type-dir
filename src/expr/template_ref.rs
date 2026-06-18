//! Shared utilities for parsing `${...}` template references.

/// Reduces a `${...}` template to its inner key, or returns the input as-is when not wrapped.
///
/// Finds the first `${` and extracts the content up to the matching `}`. When no `${...}`
/// delimiters are found the whole string is returned unchanged (plain variable name form).
pub(crate) fn strip_braces(s: &str) -> String {
    if let Some(start) = s.find("${") {
        let after = &s[start + 2..];
        if let Some(end) = after.find('}') {
            return after[..end].to_string();
        }
    }
    s.to_string()
}

/// Returns the head id when `value` is a single-segment namespaced reference `${<ns>.<id>}` for
/// any id-producing namespace (`dir`/`file`/`group`/`choice`/`use`/`for`/`fetch`).
///
/// Accepts the exact form `${<ns>.<id>}` where `<id>` is non-empty and contains no `.` or `}`
/// (no further tail hops). Returns `None` for bare references, the `value`/`with`/`rule`
/// namespaces, dotted-tail references, and composite templates. Used to recover the producer id a
/// reference targets (e.g. `${dir.q}` → `q`) when only the head id matters.
pub(crate) fn ns_head_id(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("${")?.strip_suffix('}')?;
    for prefix in &[
        "dir.", "file.", "group.", "choice.", "use.", "for.", "fetch.",
    ] {
        if let Some(id) = inner.strip_prefix(prefix) {
            if id.is_empty() || id.contains('.') || id.contains('}') {
                return None;
            }
            return Some(id);
        }
    }
    None
}

/// Returns the var segment when `key` is a `value.<var>` reference (the `value.` namespace form),
/// or the key unchanged when it has no `value.` prefix.
///
/// Used to recover the bare iteration-variable / value-binding name from a `${value.<var>}`
/// scrutinee after `strip_braces`. A dotted tail (`value.c.regex.x`) keeps only the head `c`.
pub(crate) fn value_ns_var(key: &str) -> &str {
    match key.strip_prefix("value.") {
        Some(rest) => rest.split('.').next().unwrap_or(rest),
        None => key,
    }
}

/// Returns the id segment when `value` is a `${for.<id>}` reference (the `for.` namespace form).
///
/// Accepts strings of the exact form `${for.<id>}` where `<id>` contains no `.` or `}`.
/// Returns `None` for any other form (bare references, other namespaces, composite templates).
pub(crate) fn for_ns_id(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("${")?.strip_suffix('}')?;
    let id = inner.strip_prefix("for.")?;
    if id.is_empty() || id.contains('.') || id.contains('}') {
        return None;
    }
    Some(id)
}

/// Returns the id segment when `value` is a kind-namespaced `${<ns>.<id>}` reference.
///
/// Recognizes the `choice.`, `group.`, `dir.`, and `file.` namespace prefixes. Accepts strings of
/// the exact form `${<ns>.<id>}` where `<id>` contains no `.` or `}`. Returns `None` for any other
/// form (bare references, `for.` namespace, composite templates, unknown namespaces).
pub(crate) fn kind_ns_id(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("${")?.strip_suffix('}')?;
    for prefix in &["choice.", "group.", "dir.", "file."] {
        if let Some(id) = inner.strip_prefix(prefix) {
            if id.is_empty() || id.contains('.') || id.contains('}') {
                return None;
            }
            return Some(id);
        }
    }
    None
}

/// Returns the trailing choice/group id when `value` is a `${...}` reference whose LAST hop is
/// `.choice.<id>` or `.group.<id>` (e.g. `${dir.components.choice.items}` → `items`).
///
/// Used by match-exhaustiveness to recognise a `for` source that navigates a path to a Sum
/// (one_of/any_of/choice) nested behind an id-bearing ancestor. Returns `None` when the reference
/// does not end in a choice/group hop, or when it is not a `${...}` reference at all.
pub(crate) fn tail_sum_id(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("${")?.strip_suffix('}')?;
    if inner.contains('}') {
        return None;
    }
    let segments: Vec<&str> = inner.split('.').collect();
    // Need at least `<...>.choice.<id>` (the last two segments).
    let n = segments.len();
    if n >= 2 && (segments[n - 2] == "choice" || segments[n - 2] == "group") {
        let id = segments[n - 1];
        if !id.is_empty() {
            return Some(id);
        }
    }
    None
}

/// Extracts the contents of every `${...}` reference in a template string.
///
/// Scans `template` left-to-right, collecting the text between each `${` and the next `}`. The
/// returned `Vec` preserves the order of occurrence. References that are never closed (no `}`)
/// are silently ignored.
pub(crate) fn extract_refs(template: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("${") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find('}') {
            refs.push(after[..end].to_string());
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    refs
}
