use super::super::check_rule_var_scope;
use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern};

use super::fixtures::{dir_entry, rule_with};

/// Kind-qualified reference `${value.h.regex.stem}` on a for-binding variable is allowed.
/// The for source uses `${with.handler}` (with-namespace prefix for with-params).
#[test]
fn for_binding_field_reference_is_not_an_error() {
    // Arrange: for {id: h, value: ${with.handler}} { dir: '${value.h.regex.stem}_handler' }
    let inner = dir_entry(YamlPattern::Exact(
        "${value.h.regex.stem}_handler".to_string(),
    ));
    let for_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("h".to_string()),
            source: YamlForSource::Expr("${with.handler}".to_string()),
            body: vec![inner],
        },
    };
    // handler is visible as a with-param (accessed via with. prefix)
    let r = rule_with(&["handler"], vec![for_entry]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("server_dir".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {:?}", errors);
}
