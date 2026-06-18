use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};

/// Recursively walks all rule entries and checks that with keys are within the declared range.
///
/// Keys in a reference entry's `with` must be a subset of the declared keys in the referenced
/// rule's `with_params`.
pub fn check_with_keys(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        for (index, entry) in rule.body.iter().enumerate() {
            let context = format!("rules.{}.rules[{}]", rule_name, index);
            check_entry(entry, &context, rules, &mut errors);
        }
    }
    errors
}

/// Recursively checks entries to verify with key consistency.
fn check_entry(
    entry: &YamlEntry,
    context: &str,
    rules: &IndexMap<RuleName, YamlRule>,
    errors: &mut Vec<SemanticError>,
) {
    match &entry.kind {
        YamlEntryKind::Use {
            rule: rule_name,
            with_args,
            ..
        } => {
            // If the rule is undefined, check_undefined_rule handles it, so skip here
            if let Some(rule) = rules.get(rule_name) {
                for with_key in with_args.keys() {
                    if !rule.with_params.contains_key(with_key) {
                        errors.push(SemanticError::UnknownWith {
                            rule: rule_name.to_string(),
                            with: with_key.to_string(),
                            context: context.to_string(),
                        });
                    }
                }
            }
        }
        YamlEntryKind::Choice { body, .. } => {
            for (index, alt) in body.iter().enumerate() {
                let alt_context = format!("{}.group[{}]", context, index);
                check_entry(alt, &alt_context, rules, errors);
            }
        }
        YamlEntryKind::Dir { body: children, .. } | YamlEntryKind::File { body: children, .. } => {
            if let Some(children) = children {
                for (index, child) in children.iter().enumerate() {
                    let child_context = format!("{}.rules[{}]", context, index);
                    check_entry(child, &child_context, rules, errors);
                }
            }
        }
        YamlEntryKind::Group { body: children, .. } => {
            for (index, child) in children.iter().enumerate() {
                let child_context = format!("{}.rules[{}]", context, index);
                check_entry(child, &child_context, rules, errors);
            }
        }
        YamlEntryKind::For {
            body: for_rules, ..
        } => {
            for (index, child) in for_rules.iter().enumerate() {
                let child_context = format!("{}.for.rules[{}]", context, index);
                check_entry(child, &child_context, rules, errors);
            }
        }
        YamlEntryKind::Match { body, .. } => {
            for (index, arm) in body.iter().enumerate() {
                let arm_context = format!("{}.match.rules[{}]", context, index);
                check_entry(arm, &arm_context, rules, errors);
            }
        }
        YamlEntryKind::Fetch { body } => {
            for (index, alt) in body.iter().enumerate() {
                let alt_context = format!("{}.fetch.of[{}]", context, index);
                check_entry(alt, &alt_context, rules, errors);
            }
        }
        // a value binding passes no `with:` args and owns no children
        YamlEntryKind::Value { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{RuleName, VarName, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};
    use indexmap::IndexMap;

    fn make_rule_with_with(keys: &[&str]) -> YamlRule {
        let mut with_params = IndexMap::new();
        for key in keys {
            // null = required scalar equivalent
            with_params.insert(
                VarName(key.to_string()),
                crate::yaml::YamlWithShape(serde_yaml::Value::Null),
            );
        }
        YamlRule {
            rule: RuleName("".to_string()),
            with_params,
            note: None,
            body: vec![],
        }
    }

    fn make_entry_with_with(rule_name: &str, with_keys: &[&str]) -> YamlEntry {
        let mut with_args = IndexMap::new();
        for key in with_keys {
            with_args.insert(VarName(key.to_string()), "value".to_string());
        }
        YamlEntry {
            id: None,
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::Dir {
                pattern: YamlPattern::Exact("foo".to_string()),
                body: Some(vec![YamlEntry {
                    id: None,
                    optional: None,
                    min: None,
                    max: None,
                    count: None,
                    kind: YamlEntryKind::Use {
                        rule: RuleName(rule_name.to_string()),
                        with_args,
                        colocated_rules: None,
                    },
                }]),
                colocated_use_ref: None,
            },
        }
    }

    #[test]
    fn declared_with_key_is_not_an_error() {
        // Arrange: parent_rule references s and passes a key that is declared in s's with_params
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("s".to_string()),
            make_rule_with_with(&["layer", "domain"]),
        );
        rules.insert(
            RuleName("parent_rule".to_string()),
            YamlRule {
                rule: RuleName("parent_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![make_entry_with_with("s", &["layer"])],
            },
        );

        // Act
        let errors = check_with_keys(&rules);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn undeclared_with_key_is_error() {
        // Arrange: parent_rule references s and passes a key not declared in s's with_params
        let mut rules = IndexMap::new();
        rules.insert(RuleName("s".to_string()), make_rule_with_with(&["layer"]));
        rules.insert(
            RuleName("parent_rule".to_string()),
            YamlRule {
                rule: RuleName("parent_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![make_entry_with_with("s", &["unknown_key"])],
            },
        );

        // Act
        let errors = check_with_keys(&rules);

        // Assert
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UnknownWith {
                rule,
                with,
                context,
            } => {
                assert_eq!(rule, "s");
                assert_eq!(with, "unknown_key");
                assert_eq!(context, "rules.parent_rule.rules[0].rules[0]");
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn empty_with_is_never_an_error() {
        // Arrange: parent_rule references s with an empty with
        let mut rules = IndexMap::new();
        rules.insert(RuleName("s".to_string()), make_rule_with_with(&["layer"]));
        rules.insert(
            RuleName("parent_rule".to_string()),
            YamlRule {
                rule: RuleName("parent_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![make_entry_with_with("s", &[])],
            },
        );

        // Act
        let errors = check_with_keys(&rules);

        // Assert
        assert!(errors.is_empty());
    }
}
