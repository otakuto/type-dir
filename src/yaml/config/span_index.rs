#[cfg(test)]
#[path = "span_index_tests/tests.rs"]
mod tests;

use indexmap::IndexMap;
use marked_yaml::types::MarkedMappingNode;
use marked_yaml::{Node, parse_yaml};

/// Byte-range span derived from a YAML source position.
///
/// `start` and `end` are byte offsets into the original source string.
/// `end` may be equal to `start` when only a start marker is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

/// Index mapping structural paths to their byte-range spans in the YAML source.
///
/// Keys follow the same format used by the check context strings, e.g.:
/// - `rules` — the rules mapping node
/// - `rules.<rule>` — a specific rule's mapping node
/// - `rules.<rule>.rules[<i>]` — entry at index i
/// - `rules.<rule>.rules[<i>].rules[<j>]` — nested inline entry
/// - `rules.<rule>.rules[<i>].group[<j>]` — group alternative
/// - `rules.<rule>.rules[<i>].for.rules[<j>]` — for-loop child
/// - `rules.<rule>.rules[<i>].match.rules[<j>]` — match arm
/// - `rules.<rule>.rules[<i>].fetch.of[<j>]` — fetch alternative
#[derive(Debug, Default)]
pub struct SpanIndex {
    inner: IndexMap<String, SourceSpan>,
}

impl SpanIndex {
    /// Looks up `path` exactly, then strips trailing segments one at a time
    /// until a match is found or the path is exhausted.
    ///
    /// This allows callers to find the nearest enclosing span when the exact
    /// leaf path has no entry (e.g. the field-level key does not exist in the
    /// index but its parent entry-level path does).
    pub fn lookup_with_ancestors(&self, path: &str) -> Option<SourceSpan> {
        // Exact match first.
        if let Some(span) = self.inner.get(path) {
            return Some(*span);
        }
        // Walk up by stripping the last segment.
        let mut current = path;
        loop {
            let parent = strip_last_segment(current)?;
            if let Some(span) = self.inner.get(parent) {
                return Some(*span);
            }
            current = parent;
        }
    }

    /// Returns the span covering the whole body (depth-1 direct entries) of `rules.<rule>.rules[N]`.
    /// Prefix-scans keys ending in `rules.<rule>.rules[N]` (where `]` is the final char, i.e. a
    /// depth-1 entry) and returns the span from the minimum start to the maximum end. Returns `None`
    /// when no such entry exists.
    pub fn lookup_rule_body_span(&self, rule_name: &str) -> Option<SourceSpan> {
        let prefix = format!("rules.{rule_name}.rules[");
        let mut min_start: Option<usize> = None;
        let mut max_end: Option<usize> = None;
        for (key, span) in &self.inner {
            if !key.starts_with(&prefix) {
                continue;
            }
            // Depth-1 test: `rules[N]` with nothing after the closing `]`.
            let suffix = &key[prefix.len()..];
            if !suffix.ends_with(']')
                || suffix[..suffix.len() - 1].contains(|c: char| !c.is_ascii_digit())
            {
                continue;
            }
            min_start = Some(min_start.map_or(span.start, |s| s.min(span.start)));
            max_end = Some(max_end.map_or(span.end, |e| e.max(span.end)));
        }
        match (min_start, max_end) {
            (Some(start), Some(end)) => Some(SourceSpan { start, end }),
            _ => None,
        }
    }

    /// Returns the `(path, span)` of each depth-1 direct entry `rules.<rule>.rules[N]` in index
    /// order. The depth-1 test matches `lookup_rule_body_span`.
    #[allow(dead_code)]
    pub fn lookup_rule_entries(&self, rule_name: &str) -> Vec<(String, SourceSpan)> {
        let prefix = format!("rules.{rule_name}.rules[");
        let mut entries: Vec<(String, SourceSpan)> = Vec::new();
        for (key, span) in &self.inner {
            if !key.starts_with(&prefix) {
                continue;
            }
            let suffix = &key[prefix.len()..];
            if !suffix.ends_with(']')
                || suffix[..suffix.len() - 1].contains(|c: char| !c.is_ascii_digit())
            {
                continue;
            }
            entries.push((key.clone(), *span));
        }
        entries
    }
}

/// Strips the last path segment, returning the parent path, or `None` if the
/// path has no more segments to strip.
///
/// Segments are separated by `.` or `[`.  Examples:
/// - `"rules.foo.rules[2]"` → `"rules.foo.rules"`  (strip `[2]`)
/// - `"rules.foo.rules"` → `"rules.foo"` (strip `.rules`)
/// - `"rules.foo"` → `"rules"` (strip `.foo`)
/// - `"rules"` → `None`
fn strip_last_segment(path: &str) -> Option<&str> {
    // Find the last occurrence of either `.` or `[`.
    let last_dot = path.rfind('.');
    let last_bracket = path.rfind('[');
    let pos = last_dot.into_iter().chain(last_bracket).max()?;
    if pos == 0 {
        return None;
    }
    Some(&path[..pos])
}

/// Parses `source` with marked-yaml and builds a `SpanIndex` mapping structural
/// path strings to byte spans.
///
/// Only `rules.*` paths are indexed (the check context strings are all under
/// `rules`).  Parse errors in marked-yaml are silently ignored; the returned
/// index will be empty or partial, causing the reporter to degrade gracefully.
pub fn build_span_index(source: &str) -> SpanIndex {
    let mut index = SpanIndex::default();

    let node = match parse_yaml(0, source) {
        Ok(n) => n,
        Err(_) => return index,
    };
    let Some(root_map) = node.as_mapping() else {
        return index;
    };

    // Build a char-index → byte-offset lookup table so that `Marker::character()` (which returns a
    // char index, not a byte offset) can be converted to a byte offset suitable for slicing into
    // `source`. Without this, multibyte sources produce spans that index into the middle of a UTF-8
    // sequence and panic / mis-slice.
    let char_to_byte: Vec<usize> = source
        .char_indices()
        .map(|(b, _)| b)
        .chain(std::iter::once(source.len()))
        .collect();

    // Index the "rules" mapping itself.
    let rules_node = match root_map.get_node("rules") {
        Some(n) => n,
        None => return index,
    };
    insert_span(&mut index, "rules", rules_node, &char_to_byte);

    // The top-level `rules:` is a sequence of `- rule: <name>` definition items (`rule:` is
    // for definitions; invocations write `- use: rule.<name>` elsewhere).
    let Some(rules_seq) = rules_node.as_sequence() else {
        return index;
    };

    for rule_item in rules_seq.iter() {
        let Some(rule_map) = rule_item.as_mapping() else {
            continue;
        };
        // The rule name is the `rule:` value of the definition item.
        let Some(rule_name) = rule_map
            .get_node("rule")
            .and_then(|n| n.as_scalar())
            .map(|s| s.as_str())
        else {
            continue;
        };
        let rule_path = format!("rules.{rule_name}");
        // Register the rule's span as the whole line containing the `rule:` name scalar, so that the
        // codespan source label highlights the entire definition line rather than just the scalar.
        let rule_name_node = rule_map
            .get_node("rule")
            .expect("rule name node must exist");
        if let Some(start_marker) = rule_name_node.span().start() {
            let offset = char_to_byte_offset(start_marker.character(), &char_to_byte);
            let (line_start, line_end) = line_bounds(source, offset);
            index.inner.insert(
                rule_path.clone(),
                SourceSpan {
                    start: line_start,
                    end: line_end,
                },
            );
        }

        // Index "::" entry block inside the rule definition.
        let Some(entries_node) = rule_map.get_node(":") else {
            continue;
        };
        let Some(entries_seq) = entries_node.as_sequence() else {
            continue;
        };
        for (i, entry_node) in entries_seq.iter().enumerate() {
            let entry_path = format!("{rule_path}.rules[{i}]");
            insert_span(&mut index, &entry_path, entry_node, &char_to_byte);
            if let Some(entry_map) = entry_node.as_mapping() {
                walk_entry_fields(entry_map, &entry_path, &mut index, &char_to_byte);
            }
        }
    }

    index
}

/// Indexes field-level keys and nested sub-entries of a single entry mapping.
///
/// Detects the entry kind by inspecting the discriminant keys present in the mapping
/// (`one_of`/`any_of`/`choice`, `for`, `match`, `fetch`) and generates the structural
/// path segments that match the check context strings.
///
/// Path conventions (must stay in sync with the check files):
/// - group alternatives → `{entry_path}.group[{i}]`
/// - `for` child rules  → `{entry_path}.for.rules[{i}]`
/// - `match` arm rules  → `{entry_path}.match.rules[{i}]`
/// - `fetch` alts       → `{entry_path}.fetch.of[{i}]`
/// - inline rules (dir/file/anonymous group) → `{entry_path}.rules[{i}]`
fn walk_entry_fields(
    entry_map: &MarkedMappingNode,
    entry_path: &str,
    index: &mut SpanIndex,
    char_to_byte: &[usize],
) {
    // Index every field-level key span first.
    for (field_key, field_val) in entry_map.iter() {
        let field_name = field_key.as_str();
        let field_path = format!("{entry_path}.{field_name}");
        insert_span(index, &field_path, field_val, char_to_byte);
    }

    // Dispatch on discriminant keys.
    // Group entry: one_of / any_of / choice.  The alternatives are accessed via:
    //   - `one_of`/`any_of` list form: discriminant value is a sequence
    //   - `one_of`/`any_of` of:mapping form: discriminant value is a mapping with `of` key
    //   - `choice`: discriminant value is a mapping with `of` key
    for disc in ["one_of", "any_of", "choice"] {
        if let Some(disc_node) = entry_map.get_node(disc) {
            // List form: `- one_of: [...]`
            if let Some(seq) = disc_node.as_sequence() {
                walk_group_alternatives(seq.iter(), entry_path, index, char_to_byte);
            }
            // of:mapping form: `- one_of: { id: ..., of: [...] }` or choice mapping
            else if let Some(disc_map) = disc_node.as_mapping()
                && let Some(of_node) = disc_map.get_node("of")
                && let Some(seq) = of_node.as_sequence()
            {
                walk_group_alternatives(seq.iter(), entry_path, index, char_to_byte);
            }
            return;
        }
    }

    // `for` entry: `- for: {id: <var>, value: ...}\n  ::: [...]`.
    // The `for` key value is a `{id, value}` map; the body `::` is a sibling key in the same mapping.
    if entry_map.get_node("for").is_some() {
        if let Some(rules_node) = entry_map.get_node(":") {
            walk_seq_as(rules_node, entry_path, "for.rules", index, char_to_byte);
        }
        return;
    }

    // `match` entry: `- match: <expr>\n  rules: [...]`.
    // The `match` key value is a scalar; `rules` is a sibling key.
    if entry_map.get_node("match").is_some() {
        if let Some(rules_node) = entry_map.get_node(":") {
            walk_seq_as(rules_node, entry_path, "match.rules", index, char_to_byte);
        }
        return;
    }

    // `fetch` entry: `- fetch:\n    id: ...\n    of: [...]`.
    if let Some(fetch_node) = entry_map.get_node("fetch") {
        if let Some(fetch_map) = fetch_node.as_mapping()
            && let Some(of_node) = fetch_map.get_node("of")
            && let Some(seq) = of_node.as_sequence()
        {
            for (i, alt) in seq.iter().enumerate() {
                let alt_path = format!("{entry_path}.fetch.of[{i}]");
                insert_span(index, &alt_path, alt, char_to_byte);
                if let Some(alt_map) = alt.as_mapping() {
                    walk_entry_fields(alt_map, &alt_path, index, char_to_byte);
                }
            }
        }
        return;
    }

    // Plain entry (dir/file/rule/anonymous group): `rules` is inline child list.
    if let Some(rules_node) = entry_map.get_node("rules") {
        walk_seq_as(rules_node, entry_path, "rules", index, char_to_byte);
    }
}

/// Walks a group's alternatives sequence (`one_of`/`any_of`/`choice`) and indexes each
/// alternative under `{entry_path}.group[{i}]`.
fn walk_group_alternatives<'a>(
    iter: impl Iterator<Item = &'a Node>,
    entry_path: &str,
    index: &mut SpanIndex,
    char_to_byte: &[usize],
) {
    for (i, alt) in iter.enumerate() {
        let alt_path = format!("{entry_path}.group[{i}]");
        insert_span(index, &alt_path, alt, char_to_byte);
        if let Some(alt_map) = alt.as_mapping() {
            walk_entry_fields(alt_map, &alt_path, index, char_to_byte);
        }
    }
}

/// Walks a sequence node as a child list using `{entry_path}.{sub_key}[{i}]` paths.
///
/// `sub_key` is e.g. `"rules"`, `"for.rules"`, or `"match.rules"`.
fn walk_seq_as(
    rules_node: &Node,
    entry_path: &str,
    sub_key: &str,
    index: &mut SpanIndex,
    char_to_byte: &[usize],
) {
    let Some(seq) = rules_node.as_sequence() else {
        return;
    };
    for (i, child) in seq.iter().enumerate() {
        let child_path = format!("{entry_path}.{sub_key}[{i}]");
        insert_span(index, &child_path, child, char_to_byte);
        if let Some(child_map) = child.as_mapping() {
            walk_entry_fields(child_map, &child_path, index, char_to_byte);
        }
    }
}

/// Returns the `[line_start, line_end)` byte range of the line containing `offset` in `source`.
///
/// `line_end` is the position just before the newline (the newline is excluded); for the final line
/// it is `source.len()`.
fn line_bounds(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    (start, end)
}

/// Converts a char index returned by `Marker::character()` to the corresponding byte offset using a
/// precomputed lookup table built from `source.char_indices()` plus a `source.len()` sentinel. Out
/// of bounds indices map to the last entry (`source.len()`).
fn char_to_byte_offset(char_idx: usize, char_to_byte: &[usize]) -> usize {
    char_to_byte
        .get(char_idx)
        .copied()
        .unwrap_or_else(|| char_to_byte.last().copied().unwrap_or(0))
}

/// Inserts the span for a node into the index under the given path key.
///
/// Only nodes with a known start marker are inserted.  When `end` is absent,
/// `end` is set equal to `start` so callers always get a valid (possibly
/// zero-length) range.
fn insert_span(index: &mut SpanIndex, path: &str, node: &Node, char_to_byte: &[usize]) {
    let span = node.span();
    if let Some(start_marker) = span.start() {
        let start = char_to_byte_offset(start_marker.character(), char_to_byte);
        let end = span
            .end()
            .map(|m| char_to_byte_offset(m.character(), char_to_byte))
            .unwrap_or(start);
        index
            .inner
            .insert(path.to_string(), SourceSpan { start, end });
    }
}
