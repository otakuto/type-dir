use crate::yaml::{RuleName, VarName};
use indexmap::IndexMap;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use super::{YamlEntry, YamlWithShape};

/// Represents a reusable rule definition. Treated as a macro for an entry sequence (content model).
///
/// Does not have `dir`/`file`/`outputs` fields (name-owning and outputs have been removed).
/// Nodes are described by the entry side; record fields are automatically collected from
/// the entry's captures. Legacy YAML format produces errors via deny_unknown_fields.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlRule {
    /// The rule's name (the `rule:` key of each `- rule: <name>` definition in `rules:`).
    /// This key identifies the rule for definitions; invocations write `- use: rule.<name>`.
    pub rule: RuleName,
    /// Parameter declarations as a `{id, type}` list (order-preserving, deserialized into an
    /// `IndexMap` keyed by `id`).
    ///
    /// YAML form (new):
    /// ```yaml
    /// with:
    ///   - id: p
    ///     type: rule.feature_dir
    ///   - id: q
    ///     type: type.string     # primitive
    ///   - id: r
    ///     type: [type.number]   # array of numbers
    ///   - id: s
    ///     type: {a: type.string, b: type.bool}  # object
    /// ```
    /// `type` is the param's shape (see `YamlWithShape`): an explicit primitive
    /// (`type.string`/`type.number`/`type.bool`), an array `[T]`, an object `{field: T}`, or a
    /// `rule.<name>` / `rule.<name>.<kind>.<name>` type reference. The former ambiguous `type: null`
    /// and the legacy map form (`with: { p: <shape> }`) are no longer accepted.
    #[serde(default, rename = "with", deserialize_with = "deserialize_with_params")]
    pub with_params: IndexMap<VarName, YamlWithShape>,
    /// Description of the rule (used in diagnostic output; shown as "rule '{name}': {note}" on violation).
    #[serde(default)]
    pub note: Option<String>,
    /// Entry block of the rule body. Written as `::` in YAML (the bare-`::` key is the single
    /// colon `:` after YAML consumes the second colon as the key/value separator).
    #[serde(default, rename = ":")]
    pub body: Vec<YamlEntry>,
}

/// One `{id, type}` element of a rule's `with:` parameter declaration list.
///
/// `id` is the parameter name; `type` is the param's shape declaration. Extra keys are rejected
/// via `deny_unknown_fields`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WithParamRepr {
    id: VarName,
    #[serde(rename = "type")]
    r#type: YamlWithShape,
}

/// Deserializes the declaration-side `with:` as a `[{id, type}]` list into an order-preserving
/// `IndexMap<VarName, YamlWithShape>`. Duplicate `id`s are rejected.
fn deserialize_with_params<'de, D>(
    deserializer: D,
) -> Result<IndexMap<VarName, YamlWithShape>, D::Error>
where
    D: Deserializer<'de>,
{
    let items = Vec::<WithParamRepr>::deserialize(deserializer)?;
    let mut map = IndexMap::with_capacity(items.len());
    for item in items {
        if map.insert(item.id.clone(), item.r#type).is_some() {
            return Err(D::Error::custom(format!(
                "duplicate `with` param id `{}` in rule declaration",
                item.id.0
            )));
        }
    }
    Ok(map)
}
