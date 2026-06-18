use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{PatternSpec, RuleName, YamlEntry, YamlEntryKind, YamlPattern, YamlRule};

/// Traverses all rule entry dir/file patterns and validates `PatternSpec` consistency.
///
/// Since name-owning has been removed and nodes are described by entries, only entry dir/file
/// patterns are checked.
///
/// Checks performed:
/// - `regex` is None → `InvalidPattern`
pub fn check_pattern(rules: &IndexMap<RuleName, YamlRule>) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    for (rule_name, rule) in rules {
        let ctx_prefix = format!("rule `{}`", rule_name.0);
        check_rule_patterns(rule, &ctx_prefix, &mut errors);
    }
    errors
}

fn check_rule_patterns(rule: &YamlRule, ctx_prefix: &str, errors: &mut Vec<SemanticError>) {
    for entry in &rule.body {
        check_entry_patterns(entry, ctx_prefix, errors);
    }
}

fn check_entry_patterns(entry: &YamlEntry, ctx_prefix: &str, errors: &mut Vec<SemanticError>) {
    match &entry.kind {
        YamlEntryKind::Dir { pattern, body, .. } | YamlEntryKind::File { pattern, body, .. } => {
            let label = match &entry.kind {
                YamlEntryKind::Dir { .. } => "entry dir",
                _ => "entry file",
            };
            check_yaml_pattern(pattern, &format!("{ctx_prefix} / {label}"), errors);
            if let Some(children) = body {
                for child in children {
                    check_entry_patterns(child, ctx_prefix, errors);
                }
            }
        }
        YamlEntryKind::Choice { body, .. } => {
            for alt in body {
                check_entry_patterns(alt, ctx_prefix, errors);
            }
        }
        YamlEntryKind::Group { body, .. } => {
            for child in body {
                check_entry_patterns(child, ctx_prefix, errors);
            }
        }
        YamlEntryKind::For { body, .. } => {
            for child in body {
                check_entry_patterns(child, ctx_prefix, errors);
            }
        }
        YamlEntryKind::Match { body, .. } => {
            for arm in body {
                check_entry_patterns(arm, ctx_prefix, errors);
            }
        }
        YamlEntryKind::Fetch { body } => {
            for alt in body {
                check_entry_patterns(alt, ctx_prefix, errors);
            }
        }
        YamlEntryKind::Use { .. } => {}
        // A value binding owns no dir/file pattern and no children.
        YamlEntryKind::Value { .. } => {}
    }
}

fn check_yaml_pattern(pattern: &YamlPattern, context: &str, errors: &mut Vec<SemanticError>) {
    let YamlPattern::Spec(spec) = pattern else {
        return;
    };
    check_pattern_spec(spec, context, errors);
}

pub fn check_pattern_spec(spec: &PatternSpec, context: &str, errors: &mut Vec<SemanticError>) {
    if spec.regex.is_none() {
        errors.push(SemanticError::InvalidPattern {
            context: context.to_string(),
            reason: "`regex` is not specified (required)".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::RegexPattern;

    fn make_spec(regex: Option<&str>) -> PatternSpec {
        PatternSpec {
            regex: regex.map(|s| RegexPattern(s.to_string())),
        }
    }

    #[test]
    fn regex_none_is_error() {
        // Arrange
        let spec = make_spec(None);
        let mut errors = Vec::new();

        // Act
        check_pattern_spec(&spec, "test context", &mut errors);

        // Assert
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SemanticError::InvalidPattern { .. }));
    }

    #[test]
    fn valid_regex_only_is_ok() {
        // Arrange
        let spec = make_spec(Some("^[a-z]+$"));
        let mut errors = Vec::new();

        // Act
        check_pattern_spec(&spec, "test context", &mut errors);

        // Assert
        assert!(errors.is_empty());
    }
}
