use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use crate::error::{CliError, ParseError};

const CODE: &str = "SY002";

/// Renders a YAML parse error to stderr via codespan, with the relevant source line and a caret.
pub fn print_parse_error(
    error: &ParseError,
    source: &str,
    config_file_name: &str,
) -> Result<(), CliError> {
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();
    let file = SimpleFile::new(config_file_name, source);

    let mut diagnostic = Diagnostic::error()
        .with_code(CODE)
        .with_message(format!("config parse error: {}", error.message))
        .with_notes(vec![format!("at: {}", error.path)]);

    if let Some(loc) = error.location {
        let start = loc.index.min(source.len());
        let end = (start + 1).min(source.len());
        diagnostic = diagnostic.with_labels(vec![
            Label::primary((), start..end).with_message(error.message.clone()),
        ]);
    }

    term::emit_to_write_style(&mut writer.lock(), &config, &file, &diagnostic)
        .map_err(|e| CliError::Io(std::io::Error::other(e)))
}

/// Converts a parse error into a JSON value (used in JSON output mode).
pub fn parse_error_to_json(error: &ParseError) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "code".to_string(),
        serde_json::Value::String(CODE.to_owned()),
    );
    obj.insert(
        "message".to_string(),
        serde_json::Value::String(error.message.clone()),
    );
    obj.insert(
        "path".to_string(),
        serde_json::Value::String(error.path.clone()),
    );
    if let Some(loc) = error.location {
        obj.insert("line".to_string(), serde_json::Value::from(loc.line));
        obj.insert("column".to_string(), serde_json::Value::from(loc.column));
    }
    serde_json::Value::Object(obj)
}
