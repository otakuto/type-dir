use super::super::check_rule_var_scope;
use crate::yaml::{
    EntryId, RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern,
};
use indexmap::IndexMap;

use super::fixtures::rule_with;

/// A `for` entry with `id: Y` makes `${for.Y}` a valid reference from a sibling splice with-arg.
///
/// Rule: one `for` entry (id=classified), one `splice` that references `${for.classified}` in
/// its with-args. `classified` is a self-owned id (for+id), so the reference must be valid.
#[test]
fn for_ns_bound_id_ok() {
    // Arrange: a for entry with id: classified
    let for_entry = YamlEntry {
        id: Some(EntryId("classified".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("n".to_string()),
            source: YamlForSource::Literal(vec!["a".to_string()]),
            body: vec![YamlEntry {
                id: None,
                optional: None,
                min: None,
                max: None,
                count: None,
                kind: YamlEntryKind::Dir {
                    pattern: YamlPattern::Exact("src".to_string()),
                    body: None,
                    colocated_use_ref: None,
                },
            }],
        },
    };
    // A sibling splice that references ${for.classified} in a with-arg
    let splice_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RuleName("flatten".to_string()),
            with_args: {
                let mut m = IndexMap::new();
                m.insert(
                    VarName("nodes".to_string()),
                    "${for.classified}".to_string(),
                );
                m
            },
            colocated_rules: None,
        },
    };
    let r = rule_with(&[], vec![for_entry, splice_entry]);
    let mut rules = IndexMap::new();
    rules.insert(RuleName("root".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}
