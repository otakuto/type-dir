use thiserror::Error;

/// Details of a YAML parse failure (structural path, message, and source position).
#[derive(Debug)]
pub struct ParseError {
    /// Structural path to the failing node (e.g. `rules[0].::[1]`).
    pub path: String,
    /// The message body returned by the YAML library (the trailing ` at line N column M` is stripped).
    pub message: String,
    /// Byte offset, line, and column in the source (only when provided by the library).
    pub location: Option<ParseLocation>,
}

/// A position in the source (all fields originate from `serde_yaml::Location`).
#[derive(Debug, Clone, Copy)]
pub struct ParseLocation {
    /// Zero-based byte offset.
    pub index: usize,
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at {}", self.message, self.path)?;
        if let Some(loc) = self.location {
            write!(f, ", line {} column {}", loc.line, loc.column)?;
        }
        write!(f, ")")
    }
}

/// A failure to read or parse the `.dir-lint.yaml` source itself (the lex/parse layer).
///
/// Covers I/O while reading the config file and YAML deserialization failures (serde_yaml /
/// marked-yaml). The underlying YAML errors are kept as their rendered message string so that
/// this leaf crate stays free of YAML-library dependencies.
#[derive(Debug, Error)]
pub enum SyntaxError {
    /// Failed to read the config file.
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    /// The config text is not valid YAML / does not match the expected shape.
    #[error("config parse error: {0}")]
    Parse(ParseError),
}

impl SyntaxError {
    /// Returns the diagnostic code (`SY001`..).
    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            SyntaxError::Io(_) => "SY001",
            SyntaxError::Parse(_) => "SY002",
        }
    }
}
