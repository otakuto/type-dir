use thiserror::Error;

use crate::error::cli_error::CliError;
use crate::error::lint_error::LintError;
use crate::error::runtime_error::RuntimeError;
use crate::error::semantic_error::SemanticError;
use crate::error::syntax_error::SyntaxError;

/// Top-level error for the whole dir-lint pipeline — the umbrella over every layer.
///
/// Single-failure layers (`Syntax`/`Runtime`/`Cli`) convert in via `#[from]`. The batch layers
/// (`Semantic` rule errors from `compile`, `Lint` structure findings from `check`) carry a `Vec`
/// because a single run reports many at once; the CLI renders each item, then returns the batch
/// as the process-level error whose `Display` is a one-line summary.
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    #[error("{} rule definition error(s) found", .0.len())]
    Semantic(Vec<SemanticError>),

    #[error(transparent)]
    Runtime(#[from] RuntimeError),

    #[error("{} directory structure error(s) found", .0.len())]
    Lint(Vec<LintError>),

    #[error(transparent)]
    Cli(#[from] CliError),
}
