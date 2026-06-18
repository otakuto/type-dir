use thiserror::Error;

/// An error that arises while *executing* valid rules against a directory tree (the runtime layer).
///
/// These are operational failures of the check engine itself — not lint findings (`LintError`) and
/// not rule-definition errors (`SemanticError`). `Internal` represents an invariant that the engine
/// expected to hold; it is surfaced as a recoverable error instead of panicking.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// An `ignore:` glob failed to compile while preparing the run.
    #[error("invalid ignore glob: {0}")]
    InvalidIgnoreGlob(String),

    /// An engine invariant did not hold during evaluation.
    #[error("internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),
}

impl RuntimeError {
    /// Returns the diagnostic code (`RT001`..).
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeError::InvalidIgnoreGlob(_) => "RT001",
            RuntimeError::Internal(_) => "RT002",
        }
    }
}
