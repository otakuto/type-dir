use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};

/// Recursively walks all rule entries and returns an error if a referenced rule is undefined.
pub fn check_undefined_rule(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        for (index, entry) in rule.body.iter().enumerate() {
            let context = format!("rules.{}.rules[{}]", rule_name, index);
            check_entry(entry, &context, rules, &mut errors);
        }
    }
    errors
}

/// Recursively checks entries and collects undefined rule references.
fn check_entry(
    entry: &YamlEntry,
    context: &str,
    rules: &IndexMap<RuleName, YamlRule>,
    errors: &mut Vec<SemanticError>,
) {
    match &entry.kind {
        YamlEntryKind::Use {
            rule: rule_name, ..
        } => {
            if !rules.contains_key(rule_name) {
                errors.push(SemanticError::UndefinedRule {
                    name: rule_name.to_string(),
                    context: context.to_string(),
                });
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
        // A value binding references no rule and owns no children.
        YamlEntryKind::Value { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{RuleName, YamlEntry, YamlEntryKind, YamlRule};
    use indexmap::IndexMap;

    fn make_entry_with_rule(name: &str) -> YamlEntry {
        YamlEntry {
            id: None,
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::Use {
                rule: RuleName(name.to_string()),
                with_args: IndexMap::new(),
                colocated_rules: None,
            },
        }
    }

    #[test]
    fn defined_rule_reference_is_not_an_error() {
        // Arrange: parent_rule references my_rule and my_rule is defined
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("my_rule".to_string()),
            YamlRule {
                rule: RuleName("my_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![],
            },
        );
        rules.insert(
            RuleName("parent_rule".to_string()),
            YamlRule {
                rule: RuleName("parent_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![make_entry_with_rule("my_rule")],
            },
        );

        // Act
        let errors = check_undefined_rule(&rules);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn undefined_rule_reference_is_error() {
        // Arrange: parent_rule references nonexistent but it is not defined
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("parent_rule".to_string()),
            YamlRule {
                rule: RuleName("parent_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![make_entry_with_rule("nonexistent")],
            },
        );

        // Act
        let errors = check_undefined_rule(&rules);

        // Assert
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedRule { name, context } => {
                assert_eq!(name, "nonexistent");
                assert_eq!(context, "rules.parent_rule.rules[0]");
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn nested_entries_undefined_reference_is_detected() {
        // Arrange: parent_rule's inline entries reference missing_child
        let child = make_entry_with_rule("missing_child");
        let parent_entry = YamlEntry {
            id: None,
            optional: None,
            min: None,
            max: None,
            count: None,
            kind: YamlEntryKind::File {
                pattern: crate::yaml::YamlPattern::Exact("foo".to_string()),
                body: Some(vec![child]),
                colocated_use_ref: None,
            },
        };
        let mut rules = IndexMap::new();
        rules.insert(
            RuleName("parent_rule".to_string()),
            YamlRule {
                rule: RuleName("parent_rule".to_string()),
                with_params: IndexMap::new(),
                note: None,
                body: vec![parent_entry],
            },
        );

        // Act
        let errors = check_undefined_rule(&rules);

        // Assert
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedRule { name, context } => {
                assert_eq!(name, "missing_child");
                assert_eq!(context, "rules.parent_rule.rules[0].rules[0]");
            }
            _ => panic!("unexpected error variant"),
        }
    }
}
