#[cfg(test)]
#[path = "yaml_with_shape_tests/tests.rs"]
mod tests;

use crate::yaml::{EntryId, RuleName, VarName, WithShape};
use indexmap::IndexMap;
use serde::Deserialize;

/// YAML representation of one `with` parameter `type:` declaration.
///
/// Syntax (explicit type system):
/// - `type.string` / `type.number` / `type.bool` → primitive `String` / `Number` / `Bool`.
/// - `[T]` (a one-element YAML sequence) → `Array(T)`. Nested as `[[type.number]]` for 2-D arrays.
/// - `{field: T, ...}` (a YAML mapping) → `Object(fields)`, order-preserving.
/// - `rule.<rule_name>` → non-terminal rule type reference (`RuleType { rule, path: [] }`).
/// - `rule.<rule_name>.<kind>.<name>...` → rule type with path. Each path segment is a
///   `<kind>.<name>` pair where kind ∈ dir/file/regex (`rule.R.dir.node`). Only the names are kept
///   in `path`; kinds are required syntactically but not used for resolution.
///
/// The former ambiguous `null` (scalar) and `[]` (records) forms are no longer accepted: declare an
/// explicit primitive (e.g. `type.string`) instead. The legacy `{ default: 'v' }` map form is also
/// rejected (a mapping is now an object type, and `default` is no longer a reserved key).
/// Received via `serde_yaml::Value` and converted to AST `WithShape` by `to_shape`.
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct YamlWithShape(pub serde_yaml::Value);

/// Parse error for a shape declaration. Returned as a string message for ease of use on the compile side.
#[derive(Debug, Clone)]
pub struct WithShapeError(pub String);

impl YamlWithShape {
    /// Converts a YAML value to a `WithShape`. Returns `Err` for an invalid shape.
    pub fn to_shape(&self) -> Result<WithShape, WithShapeError> {
        value_to_shape(&self.0)
    }
}

fn value_to_shape(value: &serde_yaml::Value) -> Result<WithShape, WithShapeError> {
    match value {
        // null → no longer a valid type (the ambiguous scalar encoding is removed).
        serde_yaml::Value::Null => Err(WithShapeError(
            "`type: null` is no longer supported; declare an explicit type such as \
             `type.string`, `type.number`, `type.bool`, `[T]`, `{field: T}`, or `rule.<name>`"
                .to_string(),
        )),
        // string → either a `type.<prim>` primitive or a `rule.<name>...` reference.
        serde_yaml::Value::String(s) => parse_type_string(s),
        // mapping → object type `{field: T, ...}`.
        serde_yaml::Value::Mapping(m) => parse_object(m),
        // sequence → array type `[T]`: exactly one element, itself a type.
        serde_yaml::Value::Sequence(seq) => parse_array(seq),
        _ => Err(WithShapeError(
            "with `type` must be one of: `type.string`/`type.number`/`type.bool`, `[T]` (array), \
             `{field: T}` (object), or `rule.<name>` (rule type reference)"
                .to_string(),
        )),
    }
}

/// Parses a string-valued `type:` declaration: either a `type.<prim>` primitive or a `rule.<name>`
/// reference. Any other bare string is rejected.
fn parse_type_string(s: &str) -> Result<WithShape, WithShapeError> {
    match s {
        "type.string" => return Ok(WithShape::String),
        "type.number" => return Ok(WithShape::Number),
        "type.bool" => return Ok(WithShape::Bool),
        _ => {}
    }
    if s.starts_with("rule.") || s == "rule" {
        return parse_rule_type_string(s);
    }
    if let Some(rest) = s.strip_prefix("type.") {
        return Err(WithShapeError(format!(
            "unknown primitive type `type.{rest}`; expected `type.string`, `type.number`, or `type.bool`"
        )));
    }
    Err(WithShapeError(format!(
        "invalid `type` string `{s}`; expected a primitive (`type.string`/`type.number`/`type.bool`) \
         or a rule type reference (`rule.<name>`)"
    )))
}

/// Parses a one-element YAML sequence as an array type `[T]`. The single element is recursively
/// parsed as the element type. Empty or multi-element sequences are rejected.
fn parse_array(seq: &[serde_yaml::Value]) -> Result<WithShape, WithShapeError> {
    let [elem] = seq else {
        return Err(WithShapeError(format!(
            "array type must be written as a one-element sequence `[T]` describing the element type \
             (got {} elements)",
            seq.len()
        )));
    };
    let inner = value_to_shape(elem)?;
    Ok(WithShape::Array(Box::new(inner)))
}

/// Parses a YAML mapping as an object type `{field: T, ...}`. Each value is recursively parsed as a
/// field type; keys must be strings.
fn parse_object(map: &serde_yaml::Mapping) -> Result<WithShape, WithShapeError> {
    let mut fields: IndexMap<VarName, WithShape> = IndexMap::with_capacity(map.len());
    for (k, v) in map {
        let serde_yaml::Value::String(name) = k else {
            return Err(WithShapeError(
                "object type field name must be a string".to_string(),
            ));
        };
        let field_shape = value_to_shape(v)?;
        if fields.insert(VarName(name.clone()), field_shape).is_some() {
            return Err(WithShapeError(format!(
                "duplicate object field `{name}` in `type` declaration"
            )));
        }
    }
    Ok(WithShape::Object(fields))
}

/// Parses a string value as a `RuleType` declaration.
///
/// The string must start with the `rule.` prefix. The next segment is the rule name; any
/// additional segments form the path as `<kind>.<name>` pairs (kind ∈ dir/file/regex), mirroring
/// the reference grammar (`rule.R.dir.node`). Only the names are kept in `path`; the kinds are
/// required syntactically but not used for resolution (`resolve_chain` looks up `children[name]`
/// without consulting the kind). For example:
/// - `rule.feature_dir` → `RuleType { rule: "feature_dir", path: [] }`
/// - `rule.rec_node.dir.node` → `RuleType { rule: "rec_node", path: ["node"] }`
/// - `rule.X.dir.a.file.b` → `RuleType { rule: "X", path: ["a", "b"] }`
fn parse_rule_type_string(s: &str) -> Result<WithShape, WithShapeError> {
    let rest = s.strip_prefix("rule.").ok_or_else(|| {
        WithShapeError(format!(
            "rule type reference must start with `rule.` (got `{s}`); \
             use `rule.<rule_name>` or `rule.<rule_name>.<kind>.<name>` instead of a bare name"
        ))
    })?;

    // rest is now "<rule_name>" or "<rule_name>.<kind>.<name>..."
    let mut parts = rest.splitn(2, '.');
    let rule_part = parts.next().unwrap_or("").to_string();
    let path: Vec<EntryId> = match parts.next() {
        None => vec![],
        Some(tail) => parse_path_pairs(tail, s)?,
    };

    Ok(WithShape::RuleType {
        rule: RuleName(rule_part),
        path,
    })
}

/// Parses the tail of a rule type reference into a list of path names.
///
/// The tail must be a sequence of `<kind>.<name>` pairs (kind ∈ dir/file/regex). The kind is
/// required syntactically but only the name is kept (it mirrors the reference grammar
/// `rule.R.dir.node`, where `resolve_chain` looks up `children[name]` regardless of the kind).
fn parse_path_pairs(tail: &str, full: &str) -> Result<Vec<EntryId>, WithShapeError> {
    let segments: Vec<&str> = tail.split('.').collect();
    if segments.len() % 2 != 0 {
        return Err(WithShapeError(format!(
            "rule type path in `{full}` must be a sequence of `<kind>.<name>` pairs \
             where kind is dir/file/regex (got an odd number of segments after the rule name)"
        )));
    }
    let mut path = Vec::with_capacity(segments.len() / 2);
    for pair in segments.chunks_exact(2) {
        let [kind, name] = pair else {
            unreachable!("chunks_exact(2) yields slices of length 2");
        };
        if !matches!(*kind, "dir" | "file" | "regex") {
            return Err(WithShapeError(format!(
                "rule type path segment in `{full}` must be `<kind>.<name>` where kind is \
                 dir/file/regex (got kind `{kind}`)"
            )));
        }
        path.push(EntryId((*name).to_string()));
    }
    Ok(path)
}
