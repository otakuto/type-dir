use crate::yaml::{EntryId, RuleName, ValueExpr, VarName};
use indexmap::IndexMap;
use serde::de::Error as _;
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

/// Deserialization form for the `use:` field of a splice entry.
///
/// The value must have the form `rule.<name>`. The `rule.` prefix is stripped to extract
/// the rule name. Any other prefix (or absence of `rule.`) is rejected with a clear error.
///
/// # Example
///
/// ```yaml
/// - use: rule.crate_dir
/// ```
pub struct UseRefValue(pub(crate) RuleName);

impl<'de> Deserialize<'de> for UseRefValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let name = raw.strip_prefix("rule.").ok_or_else(|| {
            D::Error::custom(format!(
                "rule invocation `use:` value must be `rule.<name>` (got `{raw}`)"
            ))
        })?;
        if name.is_empty() {
            return Err(D::Error::custom(
                "rule invocation `use:` value must be `rule.<name>` (rule name cannot be empty)",
            ));
        }
        Ok(UseRefValue(RuleName(name.to_owned())))
    }
}

use super::{YamlEntry, YamlForSource};
use crate::yaml::config::YamlPattern;

/// Deserialization representation of `YamlEntry`. One of: group, for, match, fetch, or plain entry.
///
/// `#[serde(untagged)]` is not used because untagged silently discards extra keys alongside
/// discriminant keys. Instead, the value is first received as a `serde_yaml::Mapping`, then
/// dispatched based on the presence of a discriminant key (`one_of`/`any_of`/`choice`/`for`/
/// `match`/`fetch`), and re-deserialized into the matching repr struct.
///
/// Groups (one_of/any_of/choice/fetch) are written with the discriminant as a flag and their
/// sub-entries (alternatives) under `::`; `id` (and `choice`'s `min`/`max`) are sibling keys:
/// ```yaml
/// - one_of:
///   id: x        # optional
///   ::
///     - <alt>
/// - choice:
///   min: 1
///   max: 2
///   ::
///     - <alt>
/// ```
pub enum YamlEntryRepr {
    /// `- one_of: / id?: / ::: [...]` exactly-one group.
    OneOf {
        id: Option<EntryId>,
        body: Vec<YamlEntry>,
    },
    /// `- any_of: / id?: / ::: [...]` one-or-more group.
    AnyOf {
        id: Option<EntryId>,
        body: Vec<YamlEntry>,
    },
    /// `- choice: / id?: / min?: / max?: / ::: [...]` cardinality-bounded selection.
    Choice {
        id: Option<EntryId>,
        min: usize,
        max: Option<usize>,
        body: Vec<YamlEntry>,
    },
    /// `- for: {id: <itervar>, value: <source>} / id?: <loop_id> / ::: [...]` iteration entry.
    For {
        var: VarName,
        source: YamlForSource,
        /// Optional id. When set, each binding's collected body is wrapped in one `Record` and
        /// accumulated as a `RecordList` under `out[id]`. When absent, body ids are collected
        /// transparently into the outer output.
        id: Option<EntryId>,
        body: Vec<YamlEntry>,
    },
    /// `- match: ${c} / ::: [...]` Sum elimination entry.
    Match {
        scrutinee: String,
        body: Vec<YamlEntry>,
    },
    /// `- fetch: / id: <id> / ::: [...]` non-consuming observation entry.
    Fetch { id: EntryId, body: Vec<YamlEntry> },
    /// `- id: <var> / value: ...` value variable binding entry.
    Value { var: VarName, value: ValueExpr },
    /// Normal entry.
    Plain(Box<PlainEntry>),
}

/// Deserialization form for a `value:` field: either a string scalar or a list of strings.
///
/// `#[serde(untagged)]` distinguishes a YAML scalar (`value: 'abc'`) from a sequence
/// (`value: ['a', 'b']`). Both are converted into `ValueExpr`.
#[derive(Deserialize)]
#[serde(untagged)]
enum YamlValueExpr {
    /// `value: 'abc'` — a single string.
    Scalar(String),
    /// `value: ['a', 'b']` — a list of strings.
    List(Vec<String>),
}

impl From<YamlValueExpr> for ValueExpr {
    fn from(repr: YamlValueExpr) -> ValueExpr {
        match repr {
            YamlValueExpr::Scalar(s) => ValueExpr::Scalar(s),
            YamlValueExpr::List(v) => ValueExpr::List(v),
        }
    }
}

/// `- id: <var> / value: ...` (extra/missing fields are errors via deny_unknown_fields).
///
/// `id` is required (it is the bound variable name) and only `id` and `value` are permitted;
/// any other key (dir/file/rule/min/max/count/optional/...) is rejected, which keeps `value`
/// exclusive with every other entry kind.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValueRepr {
    id: VarName,
    value: YamlValueExpr,
}

/// One `{id, value}` element of a splice invocation's `with:` argument list.
///
/// `id` is the declared parameter name; `value` is the argument value. It accepts:
/// - a reference / template string (`${use.x}` / `${with.y}` / `"prefix-${id}"` / `"literal"`),
/// - a number literal (`42`) or bool literal (`true`), canonicalized to their string form so they
///   are matched as scalars at runtime and type-checked against the declared primitive.
///
/// Array literals (`[...]`) and mapping literals (`{...}`) are not written inline here: build them
/// with a `value:` binding or pass them by reference (`value: ${...}`). Extra keys are rejected via
/// `deny_unknown_fields`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WithArgRepr {
    id: VarName,
    value: serde_yaml::Value,
}

/// Canonicalizes a `with` arg literal `serde_yaml::Value` into its scalar string form.
///
/// Strings pass through verbatim (they may be references/templates). Numbers and booleans are
/// rendered to their textual form. Null, sequences, and mappings are rejected: inline composite
/// literals are not supported on the call side (use a `value:` binding or a reference).
fn with_arg_value_to_string(value: serde_yaml::Value, id: &str) -> Result<String, String> {
    match value {
        serde_yaml::Value::String(s) => Ok(s),
        serde_yaml::Value::Number(n) => Ok(n.to_string()),
        serde_yaml::Value::Bool(b) => Ok(b.to_string()),
        serde_yaml::Value::Null => Err(format!(
            "`with` arg `{id}` value must not be null; pass a string, number, bool, or `${{...}}` reference"
        )),
        serde_yaml::Value::Sequence(_) | serde_yaml::Value::Mapping(_) => Err(format!(
            "`with` arg `{id}` value cannot be an inline array/object literal; \
             build it with a `value:` binding or pass it by reference (`value: ${{...}}`)"
        )),
        serde_yaml::Value::Tagged(_) => {
            Err(format!("`with` arg `{id}` value has an unsupported tag"))
        }
    }
}

/// Deserializes the invocation-side `with:` as a `[{id, value}]` list into an order-preserving
/// `IndexMap<VarName, String>`. Duplicate `id`s are rejected.
fn deserialize_with_args<'de, D>(deserializer: D) -> Result<IndexMap<VarName, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let items = Vec::<WithArgRepr>::deserialize(deserializer)?;
    let mut map = IndexMap::with_capacity(items.len());
    for item in items {
        let value = with_arg_value_to_string(item.value, &item.id.0).map_err(D::Error::custom)?;
        if map.insert(item.id.clone(), value).is_some() {
            return Err(D::Error::custom(format!(
                "duplicate `with` arg id `{}` in rule invocation",
                item.id.0
            )));
        }
    }
    Ok(map)
}

/// Inner map of a `for:` entry (`for: {id: <itervar>, value: <source>}`).
///
/// `id` is the iteration variable name (bound under the `value` namespace inside the body),
/// and `value` is the iteration source (a `${...}` expression or a literal list). Both are
/// required; extra keys are rejected via deny_unknown_fields.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForSpec {
    id: VarName,
    value: YamlForSource,
}

/// `- for: {id: <itervar>, value: <source>} / id?: <loop_id> / ::: [...]`
/// (extra/missing fields are errors via deny_unknown_fields).
///
/// The `for:` value is a map `{id, value}`: `id` is the iteration variable, `value` is the source.
/// The sibling `id:` (if present) is the for-block's own collection id, referenced externally via
/// `${for.<loop_id>...}` (`RefHead::ForNs`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForRepr {
    #[serde(rename = "for")]
    spec: ForSpec,
    #[serde(default)]
    id: Option<EntryId>,
    #[serde(rename = ":")]
    body: Vec<YamlEntry>,
}

/// `- match: ${c} / ::: [...]` (extra/missing fields are errors via deny_unknown_fields).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchRepr {
    #[serde(rename = "match")]
    scrutinee: String,
    #[serde(rename = ":")]
    body: Vec<YamlEntry>,
}

/// `- one_of: / id?: / ::: [...]`. The `one_of` flag value is ignored.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OneOfRepr {
    // Present only so `deny_unknown_fields` accepts the discriminant flag key; its value is ignored.
    #[allow(dead_code)]
    one_of: IgnoredAny,
    #[serde(default)]
    id: Option<EntryId>,
    #[serde(rename = ":")]
    body: Vec<YamlEntry>,
}

/// `- any_of: / id?: / ::: [...]`. The `any_of` flag value is ignored.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnyOfRepr {
    #[allow(dead_code)]
    any_of: IgnoredAny,
    #[serde(default)]
    id: Option<EntryId>,
    #[serde(rename = ":")]
    body: Vec<YamlEntry>,
}

/// `- choice: / id?: / min?: / max?: / ::: [...]`. The `choice` flag value is ignored.
/// Omitting `min` defaults to 0; omitting `max` (null) means unbounded (∞).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChoiceRepr {
    #[allow(dead_code)]
    choice: IgnoredAny,
    #[serde(default)]
    id: Option<EntryId>,
    #[serde(default)]
    min: usize,
    #[serde(default)]
    max: Option<usize>,
    #[serde(rename = ":")]
    body: Vec<YamlEntry>,
}

/// `- fetch: / id: <id> / ::: [...]`. The `fetch` flag value is ignored; `id` is required.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FetchRepr {
    #[allow(dead_code)]
    fetch: IgnoredAny,
    id: EntryId,
    #[serde(rename = ":")]
    body: Vec<YamlEntry>,
}

/// List of discriminant keys (mappings containing any of these are treated as
/// group/for/match/fetch/value-binding).
const DISCRIMINANT_KEYS: [&str; 7] = [
    "one_of", "any_of", "choice", "for", "match", "fetch", "value",
];

impl<'de> Deserialize<'de> for YamlEntryRepr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First receive as a generic Value and dispatch based on discriminant key presence.
        let value = serde_yaml::Value::deserialize(deserializer)?;
        let serde_yaml::Value::Mapping(mapping) = &value else {
            // Non-mapping values (scalars, arrays) cannot be entries.
            return Err(D::Error::custom(
                "an entry must be a mapping (with keys such as dir/file/use)",
            ));
        };

        // Enumerate the discriminant keys present in the mapping.
        let present: Vec<&str> = DISCRIMINANT_KEYS
            .iter()
            .copied()
            .filter(|key| mapping.contains_key(*key))
            .collect();

        // If no discriminant key is present, re-deserialize as Plain with deny_unknown_fields.
        let Some(&discriminant) = present.first() else {
            let plain = serde_yaml::from_value::<PlainEntry>(value).map_err(D::Error::custom)?;
            return Ok(YamlEntryRepr::Plain(Box::new(plain)));
        };

        // Only one discriminant key is allowed.
        if present.len() > 1 {
            return Err(D::Error::custom(format!(
                "discriminant keys `{}` and `{}` cannot coexist (write group/for/match/fetch alone)",
                present[0], present[1],
            )));
        }

        // Re-deserialize into the matching repr struct (each uses deny_unknown_fields, so extra
        // sibling keys are rejected automatically). Alternatives live under `::`.
        match discriminant {
            "for" => {
                let repr = serde_yaml::from_value::<ForRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::For {
                    var: repr.spec.id,
                    source: repr.spec.value,
                    id: repr.id,
                    body: repr.body,
                })
            }
            "match" => {
                let repr = serde_yaml::from_value::<MatchRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::Match {
                    scrutinee: repr.scrutinee,
                    body: repr.body,
                })
            }
            "fetch" => {
                let repr = serde_yaml::from_value::<FetchRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::Fetch {
                    id: repr.id,
                    body: repr.body,
                })
            }
            "value" => {
                let repr = serde_yaml::from_value::<ValueRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::Value {
                    var: repr.id,
                    value: ValueExpr::from(repr.value),
                })
            }
            "one_of" => {
                let repr = serde_yaml::from_value::<OneOfRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::OneOf {
                    id: repr.id,
                    body: repr.body,
                })
            }
            "any_of" => {
                let repr = serde_yaml::from_value::<AnyOfRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::AnyOf {
                    id: repr.id,
                    body: repr.body,
                })
            }
            "choice" => {
                let repr = serde_yaml::from_value::<ChoiceRepr>(value).map_err(D::Error::custom)?;
                Ok(YamlEntryRepr::Choice {
                    id: repr.id,
                    min: repr.min,
                    max: repr.max,
                    body: repr.body,
                })
            }
            other => Err(D::Error::custom(format!(
                "unknown discriminant key `{other}` (internal error: inconsistent with DISCRIMINANT_KEYS)",
            ))),
        }
    }
}

/// Deserializes the `group:` marker field: ignores the YAML value (which is typically null for a
/// value-less key) and always returns `true` to signal key presence. When the key is absent,
/// `#[serde(default)]` supplies `false` (the default for `bool`).
fn deserialize_group_marker<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    IgnoredAny::deserialize(deserializer)?;
    Ok(true)
}

/// Field set for a plain entry.
///
/// Count constraints are specified by four sibling keys: `optional` / `min` / `max` / `count` (scalar).
/// Map form `{min, max}` for `count` is not supported.
/// XOR constraints between keys (e.g., prohibiting coexistence of `count` and `{min,max,optional}`)
/// are validated by `check_entry_combination`.
///
/// Rule invocation (splice) uses the `use:` key with a `rule.<name>` value.
/// The legacy `rule:` key has been removed; unknown keys are rejected by `deny_unknown_fields`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlainEntry {
    #[serde(default)]
    pub id: Option<EntryId>,
    /// `group:` marker (value-less key) declaring an explicit record-intro group. Its presence
    /// (regardless of value) is the only signal used; the contents go under `::` (`rules`). When
    /// present alongside no dir/file/use, the entry becomes `YamlEntryKind::Group`.
    #[serde(
        default,
        rename = "group",
        deserialize_with = "deserialize_group_marker"
    )]
    pub group: bool,
    #[serde(default)]
    pub dir: Option<YamlPattern>,
    #[serde(default)]
    pub file: Option<YamlPattern>,
    #[serde(default)]
    pub optional: Option<bool>,
    #[serde(default)]
    pub min: Option<usize>,
    #[serde(default)]
    pub max: Option<usize>,
    #[serde(default)]
    pub count: Option<usize>,
    /// Splice target: `use: rule.<name>`. The `rule.` prefix is required.
    /// Deserialized via `UseRefValue` which strips the prefix and validates the form.
    #[serde(default, rename = "use")]
    pub use_ref: Option<UseRefValue>,
    /// Invocation-side arguments as a `{id, value}` list (order-preserving, deserialized into an
    /// `IndexMap` keyed by `id`).
    ///
    /// YAML form (new):
    /// ```yaml
    /// - use: rule.foo
    ///   with:
    ///     - id: p
    ///       value: use.xxx
    ///     - id: q
    ///       value: use.yyy.dir.zzz
    /// ```
    /// `id` is the declared param name; `value` is the argument (a `use.` namespace reference such
    /// as `use.<id>` / `use.<id>.dir.<z>`, or a literal/template string). The legacy map form
    /// (`with: { p: <value> }`) is no longer accepted.
    #[serde(default, rename = "with", deserialize_with = "deserialize_with_args")]
    pub with_args: IndexMap<VarName, String>,
    #[serde(default, rename = ":")]
    pub body: Option<Vec<YamlEntry>>,
}
