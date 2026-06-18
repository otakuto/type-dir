use std::collections::HashMap;
use std::path::Path;

use crate::error::{AppError, CliError, SyntaxError};
use crate::expr::{ConfigExpr, compile};
use crate::feature::report::{
    compile_errors_to_json, parse_error_to_json, print_errors, print_parse_error, report_to_json,
};
use crate::runtime::Checker;
use crate::runtime_impl::DirLintChecker;
use crate::walk::DirTreeSource;
use crate::walk_impl::RealDirTreeSource;
use crate::yaml::{build_span_index, load_yaml_config_str};

/// Output format for check results.
#[derive(Copy, Clone, Debug)]
pub enum OutputFormat {
    /// Emits human-readable diagnostics to stderr via codespan (default).
    Human,
    /// Emits diagnostics and dir trace as pretty JSON to stdout.
    Json,
}

/// Runs the pipeline that loads the config file and checks the directory structure.
pub fn run_check(config_path: &Path, format: OutputFormat) -> Result<(), AppError> {
    let source = std::fs::read_to_string(config_path).map_err(SyntaxError::Io)?;
    let config_file_name = config_path.display().to_string();
    let span_index = build_span_index(&source);

    let yaml_config = match load_yaml_config_str(&source) {
        Ok(c) => c,
        Err(SyntaxError::Parse(parse_err)) => {
            match format {
                OutputFormat::Human => print_parse_error(&parse_err, &source, &config_file_name)?,
                OutputFormat::Json => print_json(&parse_error_to_json(&parse_err))?,
            }
            return Err(AppError::Syntax(SyntaxError::Parse(parse_err)));
        }
        Err(other) => return Err(other.into()),
    };

    let config_expr = match compile(yaml_config) {
        Ok(c) => c,
        Err(errors) => {
            // Compile (rule-definition) errors: the note map cannot be built, so use an empty map.
            match format {
                OutputFormat::Human => print_errors(
                    &errors.0,
                    &HashMap::new(),
                    &source,
                    &config_file_name,
                    &span_index,
                )?,
                OutputFormat::Json => print_json(&compile_errors_to_json(&errors.0, &span_index))?,
            }
            return Err(AppError::Semantic(errors.0));
        }
    };

    // Build a map of rule name -> note (rule description) for diagnostic note resolution.
    let notes = build_note_map(&config_expr);

    // Use the parent of the config file as the base. If the parent is empty (config in cwd), normalize to ".".
    let base = match config_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    // FS traversal is performed via the DirTreeSource port (real FS implementation is in dir-lint-walk-impl).
    let tree = RealDirTreeSource
        .read(base, &config_expr.ignore)
        .map_err(CliError::Io)?;
    let report = DirLintChecker.check(&config_expr, &tree, base)?;

    match format {
        OutputFormat::Human => {
            if !report.errors.is_empty() {
                print_errors(
                    &report.errors,
                    &notes,
                    &source,
                    &config_file_name,
                    &span_index,
                )?;
                return Err(AppError::Lint(report.errors));
            }
        }
        OutputFormat::Json => {
            // In JSON mode, only JSON is written to stdout; nothing is written to stderr.
            print_json(&report_to_json(&report, &notes, &span_index))?;
            if !report.errors.is_empty() {
                return Err(AppError::Lint(report.errors));
            }
        }
    }

    Ok(())
}

/// Builds a map of rule name -> note from a `ConfigExpr` (rules without a note are excluded).
fn build_note_map(config: &ConfigExpr) -> HashMap<String, String> {
    config
        .rules
        .iter()
        .filter_map(|(name, rule)| {
            rule.note
                .as_ref()
                .map(|n| (name.0.to_owned(), n.to_owned()))
        })
        .collect()
}

/// Writes a JSON value to stdout as pretty-printed text.
fn print_json(value: &serde_json::Value) -> Result<(), CliError> {
    let text = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
    println!("{text}");
    Ok(())
}
