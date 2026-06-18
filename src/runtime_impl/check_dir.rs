#[cfg(test)]
#[path = "check_dir_tests/tests.rs"]
mod tests;

use std::path::Path;

use crate::error::{LintError, RuntimeError};
use crate::expr::ConfigExpr;
use crate::runtime::CheckReport;
use crate::walk::{DirTree, build_ignore_matcher};

use super::enforce::{TrialMemo, eval_node_traced};
use crate::runtime_impl::env::Scope;

/// Checks the directory structure against the configuration (single pass).
///
/// The environment is managed by Scope (Γ = ⟨Γ_lex, Γ_set⟩). Records collected by id producers
/// are overlaid into the scope as Γ_set (`bind_set`) by `extend_scope_with_node_records` at each
/// node during the enforce traversal, and passed to children via instance-scope (lexical shadowing).
///
/// `base` should be the parent directory of the configuration file (e.g., repository root).
/// `tree` should be a DirTree loaded from `base` with ignore globs applied
/// (the FS traversal is delegated to the caller's `DirTreeSource`).
///
/// The returned `CheckReport` contains diagnostics (`errors`) and a trace of rules applied to each
/// visited directory node (`dirs`, including the root).
pub fn check_dir(
    config: &ConfigExpr,
    tree: &DirTree,
    base: &Path,
) -> Result<CheckReport, RuntimeError> {
    let mut errors = Vec::new();
    let mut dirs = Vec::new();

    // Use the entries of the root rule (contents-only) pointed to by entry as the content model of base.
    let Some(root_rule) = config.rules.get(&config.entry) else {
        // Not normally reached because this is validated at compile time.
        return Ok(CheckReport { errors, dirs });
    };
    let root_entries = &root_rule.rules;

    // Check the base node as a node closed with an empty scope.
    let scope = Scope::new();
    // The trial memo for content-choice is shared across a single traversal (reused per `(node, rule, σ)`).
    let mut memo = TrialMemo::new();
    let mut produced = crate::runtime_impl::record_map::RecordMap::new();
    eval_node_traced(
        tree,
        root_entries,
        &scope,
        &config.rules,
        Path::new(""),
        &config.entry.0,
        &mut errors,
        &mut dirs,
        &mut produced,
        &mut memo,
    );

    // Post-filter: suppress MissingRequired for paths that match the ignore list
    // (allows skipping uncommitted generated files, etc.).
    if !config.ignore.is_empty() {
        let matcher = build_ignore_matcher(base, &config.ignore)
            .map_err(|e| RuntimeError::InvalidIgnoreGlob(e.to_string()))?;
        errors.retain(|e| match e {
            LintError::MissingRequired { parent, name, .. } => {
                let p = base.join(parent).join(name);
                // Match against both file and dir; if either is ignored, drop the MissingRequired.
                !(matcher.matched(&p, false).is_ignore() || matcher.matched(&p, true).is_ignore())
            }
            _ => true,
        });
    }

    Ok(CheckReport { errors, dirs })
}
