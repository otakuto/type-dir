use super::super::check_rule_var_scope;
use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern};

use super::fixtures::{dir_entry, rule_with};

/// `for x in ${with.items}` where `items` is a declared with-param produces no error.
#[test]
fn for_source_valid_input_is_not_an_error() {
    // Arrange: use ${with.items} (with-namespace prefix required for with-params)
    let inner = dir_entry(YamlPattern::Exact("${value.x}".to_string()));
    let for_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("x".to_string()),
            source: YamlForSource::Expr("${with.items}".to_string()),
            body: vec![inner],
        },
    };
    // `items` is declared as a with-param
    let r = rule_with(&["items"], vec![for_entry]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("my_rule".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
