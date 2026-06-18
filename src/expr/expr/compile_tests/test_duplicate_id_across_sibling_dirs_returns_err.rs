use crate::error::SemanticError;
use crate::expr::compile;
use crate::yaml::{EntryId, RuleName, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};
use indexmap::IndexMap;

use super::helpers::make_minimal_yaml;

#[test]
fn duplicate_id_across_sibling_dirs_returns_err() {
    // Arrange: build a rule where sibling dir entries `a` and `b` both carry the same id `x`.
    let make_dir_entry_with_id = |name: &str, id: &str| -> YamlEntry {
        YamlEntry {
            id: Some(EntryId(id.to_string())),
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::Dir {
                pattern: YamlPattern::Exact(name.to_string()),
                body: None,
                colocated_use_ref: None,
            },
        }
    };
    let child_rule = YamlRule {
        rule: RuleName("child".to_string()),
        with_params: IndexMap::new(),
        note: None,
        body: vec![
            make_dir_entry_with_id("a", "x"),
            make_dir_entry_with_id("b", "x"),
        ],
    };
    let yaml = make_minimal_yaml("child", child_rule, IndexMap::new());

    // Act
    let result = compile(yaml);

    // Assert: verify that DuplicateId { id: "x" } is present in the errors.
    let config_errors = result.unwrap_err();
    assert!(
        config_errors.0.iter().any(|e| matches!(
            e,
            SemanticError::DuplicateId { id } if id == "x"
        )),
        "expected DuplicateId for `x`, got: {:?}",
        config_errors.0
    );
}
