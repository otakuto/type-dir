#[cfg(test)]
#[path = "to_json_tests/tests.rs"]
mod tests;

use std::collections::HashMap;

use crate::error::SemanticError;
use crate::runtime::CheckReport;
use crate::yaml::SpanIndex;
use serde_json::{Value, json};

use super::reportable::Reportable;

/// Converts a single reportable diagnostic into a JSON object.
///
/// Uniform shape across both error categories: `code`, `message`, an optional `source_pos`
/// (`{"start", "end"}` byte range when a YAML span resolves for the first context path),
/// `rule_chain` (context paths array), `file`, `leaf`, `is_dir`, `applied_rule`, `entry_path`,
/// `entry_span`, `context_spans`, and — for findings whose applied rule has a note — `note`.
fn error_to_json<E: Reportable>(
    error: &E,
    notes: &HashMap<String, String>,
    span_index: &SpanIndex,
) -> Value {
    let mut obj = json!({ "code": error.code(), "message": error.message() });
    let map = obj
        .as_object_mut()
        .expect("constructed as object via json!");

    let context_paths = error.context_paths();

    // Keep `source_pos` for the first context path (backward compat).
    if let Some(span) = context_paths
        .first()
        .and_then(|path| span_index.lookup_with_ancestors(path))
    {
        map.insert(
            "source_pos".to_string(),
            json!({ "start": span.start, "end": span.end }),
        );
    }

    // Add rule_chain as the context paths array.
    if !context_paths.is_empty() {
        map.insert("rule_chain".to_string(), json!(context_paths));
    }

    // Add file, leaf, and is_dir from subject().
    if let Some((file, leaf, is_dir)) = error.subject() {
        map.insert("file".to_string(), json!(file));
        map.insert("leaf".to_string(), json!(leaf));
        map.insert("is_dir".to_string(), json!(is_dir));
    }

    // Add applied_rule.
    if let Some(rule) = error.applied_rule() {
        map.insert("applied_rule".to_string(), json!(rule));
    }

    // Add entry_path and entry_span (the violated entry's config path).
    if let Some(ep) = error.entry_path() {
        map.insert("entry_path".to_string(), json!(ep));
        if let Some(span) = span_index.lookup_with_ancestors(ep) {
            map.insert(
                "entry_span".to_string(),
                json!({ "start": span.start, "end": span.end }),
            );
        }
    }

    // Per-context-path source positions.
    let span_positions: Vec<Value> = context_paths
        .iter()
        .map(|path| {
            if let Some(span) = span_index.lookup_with_ancestors(path) {
                json!({ "path": path, "start": span.start, "end": span.end })
            } else {
                json!({ "path": path })
            }
        })
        .collect();
    if !span_positions.is_empty() {
        map.insert("context_spans".to_string(), json!(span_positions));
    }

    if let Some(rule) = error.applied_rule()
        && let Some(note) = notes.get(rule)
    {
        map.insert("note".to_string(), json!(note));
    }

    obj
}

/// Converts a slice of reportable diagnostics into a JSON array.
fn errors_to_json<E: Reportable>(
    errors: &[E],
    notes: &HashMap<String, String>,
    span_index: &SpanIndex,
) -> Value {
    Value::Array(
        errors
            .iter()
            .map(|e| error_to_json(e, notes, span_index))
            .collect(),
    )
}

/// Converts a `CheckReport` to `{ "errors": [...], "dir": [...] }` JSON.
///
/// `dir` is a trace of rules applied to each directory node (`{ "path", "rule" }`).
pub fn report_to_json(
    report: &CheckReport,
    notes: &HashMap<String, String>,
    span_index: &SpanIndex,
) -> Value {
    let dirs: Vec<Value> = report
        .dirs
        .iter()
        .map(|d| json!({ "path": d.path.display().to_string(), "rule": d.rule }))
        .collect();
    json!({ "errors": errors_to_json(&report.errors, notes, span_index), "dir": dirs })
}

/// Converts compile errors (rule-definition `SemanticError`s) to `{ "errors": [...], "dir": [] }`.
///
/// No dir trace is available at compile time, so `dir` is empty; no note map applies.
pub fn compile_errors_to_json(errors: &[SemanticError], span_index: &SpanIndex) -> Value {
    json!({ "errors": errors_to_json(errors, &HashMap::new(), span_index), "dir": [] })
}
