use std::path::Path;

use super::YamlConfig;
use crate::error::{ParseError, ParseLocation, SyntaxError};

/// Reads a YAML config file and converts it into a `YamlConfig`.
#[allow(dead_code)]
pub fn load_yaml_config(path: &Path) -> Result<YamlConfig, SyntaxError> {
    let content = std::fs::read_to_string(path)?;
    load_yaml_config_str(&content)
}

/// Converts an already-loaded YAML source string into a `YamlConfig`.
///
/// Deserializes via `serde_path_to_error`; on failure it stores the structural path
/// and source position in a `ParseError` so the diagnostic reporter can emit a caret.
pub fn load_yaml_config_str(content: &str) -> Result<YamlConfig, SyntaxError> {
    let deserializer = serde_yaml::Deserializer::from_str(content);
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let path = error.path().to_string();
        let inner = error.inner();
        let location = inner.location().map(|loc| ParseLocation {
            index: loc.index(),
            line: loc.line(),
            column: loc.column(),
        });
        let mut message = inner.to_string();
        // Strip the trailing ` at line N column M` from serde_yaml messages since it duplicates the caret.
        if let Some(loc) = &location {
            let suffix = format!(" at line {} column {}", loc.line, loc.column);
            if let Some(stripped) = message.strip_suffix(&suffix) {
                message = stripped.to_string();
            }
        }
        SyntaxError::Parse(ParseError {
            path,
            message,
            location,
        })
    })
}
