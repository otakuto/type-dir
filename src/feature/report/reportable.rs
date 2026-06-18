use crate::error::{LintError, SemanticError};

/// A renderable diagnostic: the common surface that both rule-definition errors (`SemanticError`)
/// and directory-structure findings (`LintError`) expose, so the reporting code (`print_errors`,
/// `to_json`) can render either uniformly.
pub trait Reportable {
    /// The diagnostic code (e.g. `SM001`, `LT001`).
    fn code(&self) -> &'static str;
    /// The human-readable message (the `Display` rendering).
    fn message(&self) -> String;
    /// The structural paths into the YAML config for source-span lookup.
    fn context_paths(&self) -> Vec<String>;
    /// A supplemental fix hint shown beneath the diagnostic, if any.
    fn fix_hint(&self) -> Option<&'static str>;
    /// The applied content-model rule whose note should be prepended, if any (findings only).
    fn applied_rule(&self) -> Option<&str> {
        None
    }
    /// A concise headline message (without parent/rule suffixes).
    fn headline(&self) -> String {
        self.message()
    }
    /// Returns `(full_path, leaf, is_dir)` for display, if applicable.
    fn subject(&self) -> Option<(String, String, bool)> {
        None
    }
    /// Returns the rule chain names (raw rule names, without prefix), for chain line display.
    /// Default: empty (SemanticError has no chain).
    fn rule_chain(&self) -> Vec<String> {
        vec![]
    }
    /// Returns the config entry path of the violated entry, if known.
    fn entry_path(&self) -> Option<&str> {
        None
    }
}

impl Reportable for SemanticError {
    fn code(&self) -> &'static str {
        self.code()
    }
    fn message(&self) -> String {
        self.to_string()
    }
    fn context_paths(&self) -> Vec<String> {
        self.context_path().into_iter().collect()
    }
    fn fix_hint(&self) -> Option<&'static str> {
        self.fix_hint()
    }
}

impl Reportable for LintError {
    fn code(&self) -> &'static str {
        self.code()
    }
    fn message(&self) -> String {
        self.to_string()
    }
    fn context_paths(&self) -> Vec<String> {
        self.context_paths()
    }
    fn fix_hint(&self) -> Option<&'static str> {
        self.fix_hint()
    }
    fn applied_rule(&self) -> Option<&str> {
        match self {
            LintError::Undeclared { rule, .. } | LintError::MissingRequired { rule, .. } => {
                Some(rule)
            }
            LintError::CardinalityViolation { rule_chain, .. }
            | LintError::CountViolation { rule_chain, .. } => rule_chain.last().map(|s| s.as_str()),
        }
    }
    fn headline(&self) -> String {
        self.headline()
    }
    fn subject(&self) -> Option<(String, String, bool)> {
        self.subject()
    }
    fn entry_path(&self) -> Option<&str> {
        self.entry_path()
    }
    fn rule_chain(&self) -> Vec<String> {
        self.deduped_rule_chain()
    }
}
