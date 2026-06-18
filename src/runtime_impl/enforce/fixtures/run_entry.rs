use crate::error::LintError;
use crate::expr::ExprEntry;
use crate::walk::DirTree;

use super::run_entries::run_entries;

/// Helper that runs eval_node with a single entry and an empty scope.
pub fn run_entry(entry: ExprEntry, tree: &DirTree) -> Vec<LintError> {
    run_entries(&[entry], tree)
}
