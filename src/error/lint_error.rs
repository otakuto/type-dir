use std::path::{Path, PathBuf};

use thiserror::Error;

/// A directory-structure violation found while enforcing valid rules against a real tree.
///
/// These are the lint findings (the "your files violate the rules" category), as opposed to
/// rule-definition errors (`SemanticError`) or parse errors (`SyntaxError`). `Clone` is derived so
/// that content-choice trial findings can be reused via memoization (`TrialMemo`).
///
/// The `context_path()` method returns a structural path string (e.g. `"rules.foo.rules[2]"`) usable
/// as a key into a `SpanIndex`; `None` when no config-structure position applies.
#[derive(Debug, Clone, Error)]
pub enum LintError {
    /// An existing dir/file that does not match any declared entry (deny-by-default).
    /// `rule` is the name of the content-model rule that was applied to this node.
    /// `rule_chain` is the directory-descent chain of applied rules (outermost first).
    #[error("undeclared path: {} (rule: {rule})", path.display())]
    Undeclared {
        path: PathBuf,
        is_dir: bool,
        rule: String,
        rule_chain: Vec<String>,
    },

    /// A required name that does not exist.
    /// `rule` is the content-model rule applied to this required entry (used for note resolution).
    /// `context` describes the for-binding provenance (e.g. `"f=z, ff=a"`); empty when none.
    #[error(
        "required name not found: {name} (parent: {}, rule: {rule}){}",
        parent.display(),
        context_suffix(context)
    )]
    MissingRequired {
        parent: PathBuf,
        name: String,
        is_dir: bool,
        rule: String,
        context: String,
        rule_chain: Vec<String>,
        entry_path: Option<String>,
    },

    /// The number of realized items in a group (one_of/any_of) violates the cardinality constraint.
    #[error("{}", cardinality_message(*realized, *min, *max, parent))]
    CardinalityViolation {
        parent: PathBuf,
        realized: usize,
        min: usize,
        max: Option<usize>,
        rule_chain: Vec<String>,
        entry_path: Option<String>,
    },

    /// The count constraint (`count:`) on a dir/file entry is violated.
    ///
    /// `parent` is the path of this node, `name` is the display of the entry name pattern,
    /// `observed` is the number of children assigned to the entry, and `min`/`max` is the
    /// constraint interval (`max = None` means ∞).
    #[error(
        "count constraint violated: `{name}` (observed {observed}, expected [{min}, {}], parent: {})",
        max_label(*max),
        parent.display()
    )]
    CountViolation {
        parent: PathBuf,
        name: String,
        observed: usize,
        min: usize,
        max: Option<usize>,
        rule_chain: Vec<String>,
        entry_path: Option<String>,
    },
}

impl LintError {
    /// Returns the diagnostic code (`LT001`..).
    pub fn code(&self) -> &'static str {
        match self {
            LintError::Undeclared { .. } => "LT001",
            LintError::MissingRequired { .. } => "LT002",
            LintError::CardinalityViolation { .. } => "LT003",
            LintError::CountViolation { .. } => "LT004",
        }
    }

    /// Returns the structural paths into the YAML config most closely associated with this finding.
    /// Each element corresponds to a rule in `rule_chain` and is formatted as `"rules.{r}"`.
    /// Preserves order, deduplicates adjacent identical entries, and excludes empty-string rules.
    pub fn context_paths(&self) -> Vec<String> {
        let chain = match self {
            LintError::Undeclared { rule_chain, .. }
            | LintError::MissingRequired { rule_chain, .. }
            | LintError::CardinalityViolation { rule_chain, .. }
            | LintError::CountViolation { rule_chain, .. } => rule_chain,
        };
        let mut result: Vec<String> = Vec::new();
        for r in chain {
            if r.is_empty() {
                continue;
            }
            let path = format!("rules.{r}");
            if result.last().map(|s| s.as_str()) != Some(&path) {
                result.push(path);
            }
        }
        result
    }

    /// Returns a supplemental fix hint shown beneath the diagnostic, if any.
    pub fn fix_hint(&self) -> Option<&'static str> {
        match self {
            LintError::Undeclared { .. } => Some(
                "to allow it, add `- file: <name>` or `- dir: <name>` to the rule for the parent directory. for generated artifacts, add a path glob to `ignore:`",
            ),
            LintError::MissingRequired { .. } => Some(
                "if the entry is optional, wrap it with `optional:`. if it is an uncommitted generated file, add it to `ignore:`",
            ),
            LintError::CardinalityViolation { .. } => Some(
                "one_of requires exactly 1 match; any_of requires at least 1. revise the structure or add `optional` to relax the lower bound",
            ),
            LintError::CountViolation { .. } => Some(
                "adjust the number of matching entries to fit within [min, max]. set min to 0 to allow absence",
            ),
        }
    }

    /// Returns the config path of the violated entry (e.g. `"rules.foo.rules[2]"`), if known.
    pub fn entry_path(&self) -> Option<&str> {
        match self {
            LintError::MissingRequired { entry_path, .. }
            | LintError::CountViolation { entry_path, .. }
            | LintError::CardinalityViolation { entry_path, .. } => entry_path.as_deref(),
            LintError::Undeclared { .. } => None,
        }
    }

    /// Returns the rule chain deduplicated: empty rules are skipped and adjacent identical entries
    /// are collapsed. Used by reporting code to build source-span blocks.
    pub fn deduped_rule_chain(&self) -> Vec<String> {
        let chain = match self {
            LintError::Undeclared { rule_chain, .. }
            | LintError::MissingRequired { rule_chain, .. }
            | LintError::CardinalityViolation { rule_chain, .. }
            | LintError::CountViolation { rule_chain, .. } => rule_chain,
        };
        let mut result: Vec<String> = Vec::new();
        for r in chain {
            if r.is_empty() {
                continue;
            }
            if result.last().map(|s| s.as_str()) != Some(r.as_str()) {
                result.push(r.clone());
            }
        }
        result
    }

    /// Returns a concise headline message (without parent/rule suffixes).
    pub fn headline(&self) -> String {
        match self {
            LintError::Undeclared { .. } => "undeclared path".to_owned(),
            LintError::MissingRequired { .. } => "required name not found".to_owned(),
            LintError::CountViolation {
                observed, min, max, ..
            } => {
                format!(
                    "count constraint violated (observed {observed}, expected [{min}, {}])",
                    max_label(*max)
                )
            }
            LintError::CardinalityViolation {
                realized, min, max, ..
            } => cardinality_headline(*realized, *min, *max),
        }
    }

    /// Returns `(full_path_string, leaf_name_string, is_dir)` for display, if applicable.
    pub fn subject(&self) -> Option<(String, String, bool)> {
        match self {
            LintError::Undeclared { path, is_dir, .. } => {
                let leaf = path.file_name()?.to_string_lossy().into_owned();
                Some((path.display().to_string(), leaf, *is_dir))
            }
            LintError::MissingRequired {
                parent,
                name,
                is_dir,
                ..
            } => Some((
                parent.join(name).display().to_string(),
                name.clone(),
                *is_dir,
            )),
            LintError::CountViolation { parent, .. } => {
                let leaf = parent.file_name()?.to_string_lossy().into_owned();
                Some((parent.display().to_string(), leaf, false))
            }
            LintError::CardinalityViolation { parent, .. } => {
                let leaf = parent.file_name()?.to_string_lossy().into_owned();
                Some((parent.display().to_string(), leaf, false))
            }
        }
    }
}

/// Renders the optional `(context: ...)` suffix of `MissingRequired` (empty when no provenance).
fn context_suffix(context: &str) -> String {
    if context.is_empty() {
        String::new()
    } else {
        format!(" (context: {context})")
    }
}

/// Renders the upper bound of a count/cardinality interval (`inf` when unbounded).
fn max_label(max: Option<usize>) -> String {
    match max {
        Some(m) => m.to_string(),
        None => "inf".to_owned(),
    }
}

/// Builds a concise headline for a cardinality violation (without parent path).
fn cardinality_headline(realized: usize, min: usize, max: Option<usize>) -> String {
    if realized < min {
        format!("at least one group required but only {realized} found (min: {min})")
    } else if let Some(m) = max
        && realized > m
    {
        format!("group allows at most {m} but {realized} found")
    } else {
        format!("cardinality constraint violated (realized={realized}, min={min}, max={max:?})")
    }
}

/// Builds the cardinality-violation message, which differs by which bound was crossed.
fn cardinality_message(realized: usize, min: usize, max: Option<usize>, parent: &Path) -> String {
    if realized < min {
        format!(
            "at least one group required but only {realized} found (min: {min}, parent: {})",
            parent.display()
        )
    } else if let Some(m) = max
        && realized > m
    {
        format!(
            "group allows at most {m} but {realized} found (parent: {})",
            parent.display()
        )
    } else {
        format!(
            "cardinality constraint violated (realized={realized}, min={min}, max={max:?}, parent: {})",
            parent.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::LintError;

    /// `CountViolation::subject()` returns the parent path and its leaf, not the name pattern string.
    /// Even when `name` contains a regex pattern such as `/.*/`, `PathBuf::join` does not discard the parent.
    #[test]
    fn count_violation_subject_returns_parent_not_pattern() {
        // Arrange
        let parent = PathBuf::from("/some/project/src");
        let error = LintError::CountViolation {
            parent: parent.clone(),
            name: "/.*/".to_string(),
            observed: 0,
            min: 1,
            max: None,
            rule_chain: vec![],
            entry_path: None,
        };

        // Act
        let subject = error.subject();

        // Assert
        let (file, leaf, _is_dir) = subject.expect("subject must be Some");
        assert_eq!(file, "/some/project/src");
        assert_eq!(leaf, "src");
        assert!(
            !file.contains("/.*/"),
            "file must not contain the pattern string"
        );
    }
}
