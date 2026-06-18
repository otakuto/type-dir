use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind, YamlPattern};

use super::fixtures::{dir_entry, rule_with};

/// Bare references are rejected even when the referenced id is a file inside an id-less dir.
/// Id-less dirs are opaque: inner ids do not bubble out, and bare references are always rejected.
#[test]
fn bare_reference_to_self_owned_id_is_error() {
    // Arrange: Place file id: sqlx_query inside dir: queries (no id) — bare reference is rejected
    let sqlx_file = YamlEntry {
        id: Some(EntryId("sqlx_query".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("query.sql".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let queries_dir = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("queries".to_string()),
            body: Some(vec![sqlx_file]),
            colocated_use_ref: None,
        },
    };
    // Reference bare `${sqlx_query}` from a separate dir pattern
    let consumer = dir_entry(YamlPattern::Exact("${sqlx_query}_dir".to_string()));
    let r = rule_with(&[], vec![queries_dir, consumer]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("core_query_crate_dir".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: bare reference is rejected
    assert!(
        errors.iter().any(|e| matches!(e, SemanticError::BareReference { reference, .. } if reference == "sqlx_query")),
        "expected BareReference for sqlx_query: {:?}",
        errors
    );
}
