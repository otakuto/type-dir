use thiserror::Error;

/// An error in the CLI orchestration layer (wiring the pipeline together), distinct from parsing,
/// rule validation, execution, or findings. Currently this is I/O while reading the target
/// directory tree.
#[derive(Debug, Error)]
pub enum CliError {
    /// Failed to read the target directory tree.
    #[error("failed to read directory tree: {0}")]
    Io(#[from] std::io::Error),
}

impl CliError {
    /// Returns the diagnostic code (`CL001`..).
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            CliError::Io(_) => "CL001",
        }
    }
}
