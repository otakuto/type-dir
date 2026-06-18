use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::ExprEntry;
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::walk::DirTree;

/// Helper that runs eval_node with multiple entries and an empty scope.
pub fn run_entries(entries: &[ExprEntry], tree: &DirTree) -> Vec<LintError> {
    let scope = crate::runtime_impl::env::Scope::new();
    let rules = IndexMap::new();
    let mut errors = Vec::new();
    let mut produced = crate::runtime_impl::record_map::RecordMap::new();
    eval_node(
        tree,
        entries,
        &scope,
        &rules,
        Path::new("root"),
        "test_rule",
        &mut errors,
        &mut produced,
        &mut TrialMemo::new(),
    );
    errors
}
