use super::super::check_rule_var_scope;
use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RegexPattern, RuleName, YamlEntry, YamlEntryKind, YamlPattern};

use super::fixtures::rule_with;

/// Bare references to ancestor regex captures are rejected as BareReference errors.
#[test]
fn ancestor_capture_bare_reference_is_error() {
    // Arrange: dir regex captures domain and the child file references ${domain} bare
    let child = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::File {
            pattern: YamlPattern::Exact("${domain}.rs".to_string()),
            body: None,
            colocated_use_ref: None,
        },
    };
    let parent = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Spec(PatternSpec {
                regex: Some(RegexPattern(r"^(?<domain>[a-z]+)$".to_string())),
            }),
            body: Some(vec![child]),
            colocated_use_ref: None,
        },
    };
    let r = rule_with(&[], vec![parent]);
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("s".to_string()), r);

    // Act
    let errors = check_rule_var_scope(&rules);

    // Assert: bare reference to ancestor capture is rejected
    assert_eq!(errors.len(), 1, "expected 1 error: {:?}", errors);
    let SemanticError::BareReference { reference, .. } = &errors[0] else {
        panic!("unexpected: {:?}", errors[0]);
    };
    assert_eq!(reference, "domain");
}
