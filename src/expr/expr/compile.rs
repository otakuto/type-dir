#[cfg(test)]
#[path = "compile_tests/tests.rs"]
mod tests;

mod count;
mod entry;
mod matcher;
mod pattern;
mod rule;

use rule::to_expr_rule;

use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::expr::{ConfigErrors, ConfigExpr};
use crate::yaml::{RuleName, YamlConfig, YamlRule};

/// Compiles a validated `YamlConfig` into a `ConfigExpr`.
///
/// `ConfigExpr` is defined in a separate crate (dir-lint-config), so implementing `TryFrom`
/// directly would cause a coherence violation; this is provided as a free function instead.
/// Validation errors are returned as `ConfigErrors` (a collection of `SemanticError`).
pub fn compile(yaml: YamlConfig) -> Result<ConfigExpr, ConfigErrors> {
    // Run all static checks and return all errors at once if any are found.
    let mut errors: Vec<SemanticError> = Vec::new();
    if yaml.version != 0 {
        errors.push(SemanticError::UnsupportedVersion {
            version: yaml.version,
        });
    }

    // The top-level `rules:` is a sequence of `- rule: <name>` definitions (`rule:` is the
    // definition key; invocations write `- use: rule.<name>`). Build the name→rule map used by all
    // downstream checks, reporting any duplicate rule name (first definition wins).
    let mut rules: IndexMap<RuleName, YamlRule> = IndexMap::new();
    for rule in yaml.rules {
        if rules.contains_key(&rule.rule) {
            errors.push(SemanticError::DuplicateRule {
                rule: rule.rule.0.clone(),
            });
        } else {
            rules.insert(rule.rule.clone(), rule);
        }
    }

    errors.extend(super::check::check_config_expr_yaml(&rules, &yaml.entry));
    errors.extend(super::check::check_splice_cycles(&rules));
    errors.extend(super::check::check_id_capture_required(&rules));
    errors.extend(super::check::check_capture_requires_id(&rules));
    if !errors.is_empty() {
        return Err(ConfigErrors(errors));
    }

    // After passing all checks, build the model (invalid states are assumed not to occur).
    let rules = rules
        .iter()
        .map(|(name, rule)| (name.clone(), to_expr_rule(rule)))
        .collect();

    Ok(ConfigExpr {
        ignore: yaml.ignore,
        rules,
        entry: yaml.entry,
    })
}
