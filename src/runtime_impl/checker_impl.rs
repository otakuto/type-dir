use std::path::Path;

use crate::error::RuntimeError;
use crate::expr::ConfigExpr;
use crate::runtime::{CheckReport, Checker};
use crate::walk::DirTree;

/// Checker implementation using the standard dir-lint engine.
pub struct DirLintChecker;

impl Checker for DirLintChecker {
    fn check(
        &self,
        config: &ConfigExpr,
        tree: &DirTree,
        base: &Path,
    ) -> Result<CheckReport, RuntimeError> {
        crate::runtime_impl::check_dir::check_dir(config, tree, base)
    }
}
