use super::super::check_rule_var_scope;
use crate::yaml::{
    EntryId, RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern,
};

use super::fixtures::rule_with;

/// A forward reference — consumer appears before producer in source order — is an error.
/// `for h in ${schema}` comes first in source order, but the dir with `id: schema` is declared later.
#[test]
fn forward_reference_to_later_sibling_id_is_error() {
    // Arrange: consumer (for h in ${schema}) comes first, producer (dir with id: schema) comes after
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
    // producer: dir with id: schema (appears after in source order)
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
    let r = rule_with(&[], vec![consumer, schema_file]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("root".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: ForwardReference (SM021) because `schema` is declared after the reference.
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    assert!(
        matches!(
            &errors[0],
            crate::error::SemanticError::ForwardReference { id, .. } if id == "schema"
        ),
        "unexpected: {:?}",
        errors[0]
    );
}
