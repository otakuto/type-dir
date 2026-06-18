use super::super::check_rule_var_scope;
use crate::yaml::{
    EntryId, RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern,
};

use super::fixtures::rule_with;

/// A backward reference — producer appears before consumer in source order — is not an error.
/// The dir with `id: schema` is declared first, and `for h in ${schema}` comes after.
#[test]
fn backward_reference_to_earlier_sibling_id_is_ok() {
    // Arrange: producer (dir with id: schema) comes first, consumer (for h in ${schema}) comes after
    let schema_file = YamlEntry {
        id: Some(EntryId("schema".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("schema.yaml".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let inner_file = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("${value.h}_handler.rs".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let consumer = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("h".to_string()),
            source: YamlForSource::Expr("${file.schema}".to_string()),
            body: vec![inner_file],
        },
    };
    let r = rule_with(&[], vec![schema_file, consumer]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("root".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: no error because producer precedes consumer in source order.
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
