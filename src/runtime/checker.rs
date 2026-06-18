use std::path::{Path, PathBuf};

use crate::error::{LintError, RuntimeError};
use crate::expr::ConfigExpr;
use crate::walk::DirTree;

/// Represents the rule applied to a single directory node.
///
/// `path` is relative to the repository root (root is `""` or `"."`), and
/// `rule` is the rule name as propagated by C2.
pub struct DirTrace {
    pub path: PathBuf,
    pub rule: String,
}

/// Represents the check result: diagnostics and a trace of rules applied to each directory.
pub struct CheckReport {
    pub errors: Vec<LintError>,
    pub dirs: Vec<DirTrace>,
}

/// Port that runs a check from a ConfigExpr and a DirTree (implemented by runtime-impl).
pub trait Checker {
    /// Checks `tree` (a DirTree read from `base` with ignore globs applied) according to `config`.
    ///
    /// Returns a `CheckReport` containing diagnostics and a trace of rules applied to each visited
    /// directory node.
    fn check(
        &self,
        config: &ConfigExpr,
        tree: &DirTree,
        base: &Path,
    ) -> Result<CheckReport, RuntimeError>;
}
