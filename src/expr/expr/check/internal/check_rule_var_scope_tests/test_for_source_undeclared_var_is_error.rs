use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind, YamlForSource, YamlPattern};

use super::fixtures::{dir_entry, rule_with};

/// `for x in ${typo}` where `typo` is a bare reference produces `BareReference`.
#[test]
fn for_source_undeclared_var_is_error() {
    // Arrange
    let inner = dir_entry(YamlPattern::Exact("${value.x}".to_string()));
    let for_entry = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::For {
            var: VarName("x".to_string()),
            source: YamlForSource::Expr("${typo}".to_string()),
            body: vec![inner],
        },
    };
    // no with-params declared — `typo` is undeclared
    let r = rule_with(&[], vec![for_entry]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("my_rule".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert
    assert_eq!(
        errors.len(),
        1,
        "expected exactly 1 error, got: {:?}",
        errors
    );
    let SemanticError::BareReference { rule, reference } = &errors[0] else {
        panic!("unexpected error kind: {:?}", errors[0]);
    };
    assert_eq!(rule, "my_rule");
    assert_eq!(reference, "typo");
}
