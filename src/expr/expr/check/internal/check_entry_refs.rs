use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlRule};

/// Validates the consistency of the start-symbol rule pointed to by the entry.
///
/// All rules are content-model macros (name-owning is removed), so entry may point to any
/// defined rule. The check only verifies existence.
///
/// Check performed:
/// - entry does not exist in `rules` → `UndefinedRule`
pub fn check_entry_refs(
    entry: &RuleName,
    rules: &IndexMap<RuleName, YamlRule>,
) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    if rules.get(entry).is_none() {
        errors.push(SemanticError::UndefinedRule {
            name: entry.to_string(),
            context: "entry".to_string(),
        });
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{RuleName, YamlRule};
    use indexmap::IndexMap;

    fn make_rule() -> YamlRule {
        YamlRule {
            rule: RuleName("".to_string()),
            with_params: IndexMap::new(),
            note: None,
            body: vec![],
        }
    }

    #[test]
    fn defined_rule_reference_is_not_an_error() {
        // Arrange
        let mut rules = IndexMap::new();
        rules.insert(RuleName("root".to_string()), make_rule());
        let entry = RuleName("root".to_string());

        // Act
        let errors = check_entry_refs(&entry, &rules);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn undefined_rule_reference_is_undefined_rule_error() {
        // Arrange
        let rules = IndexMap::new();
        let entry = RuleName("nonexistent".to_string());

        // Act
        let errors = check_entry_refs(&entry, &rules);

        // Assert
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            SemanticError::UndefinedRule { name, context } => {
                assert_eq!(name, "nonexistent");
                assert_eq!(context, "entry");
            }
            _ => panic!("unexpected error variant: {:?}", errors[0]),
        }
    }
}
