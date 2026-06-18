use std::collections::HashMap;
use std::io::Write;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{
    Color, ColorChoice, ColorSpec, StandardStream, WriteColor,
};

use crate::error::CliError;
use crate::yaml::SpanIndex;

use super::reportable::Reportable;

/// Prints a list of reportable diagnostics to stderr.
///
/// For errors with a non-empty rule chain (`LintError`), emits independent `-->` blocks — one per
/// rule in the chain — with the applied (last) rule underlined with `^` and ancestors connected with
/// a `>` marker. For errors without a rule chain (`SemanticError` etc.) falls back to codespan.
pub fn print_errors<E: Reportable>(
    errors: &[E],
    notes: &HashMap<String, String>,
    source: &str,
    config_file_name: &str,
    span_index: &SpanIndex,
) -> Result<(), CliError> {
    let mut writer = StandardStream::stderr(ColorChoice::Auto);
    let use_color = should_use_color();

    for error in errors {
        let chain = error.rule_chain();
        if chain.is_empty() {
            // SemanticError path: use codespan as before.
            let config = term::Config::default();
            let file = SimpleFile::new(config_file_name, source);
            let (diagnostic, gutter_w) = to_diagnostic(error, span_index, use_color, source);
            term::emit_to_write_style(&mut writer.lock(), &config, &file, &diagnostic)
                .map_err(color_err)?;

            let pad = if gutter_w == 0 {
                String::new()
            } else {
                " ".repeat(gutter_w + 1)
            };
            if let Some(rule) = error.applied_rule()
                && let Some(note) = notes.get(rule)
            {
                writeln!(writer.lock(), "{pad}= rule '{rule}': {note}").map_err(CliError::Io)?;
            }
            if let Some(hint) = error.fix_hint() {
                writeln!(writer.lock(), "{pad}= {hint}").map_err(CliError::Io)?;
            }
        } else {
            // LintError path: draw manual --> blocks.
            print_chain_blocks(
                &mut writer,
                error,
                &chain,
                notes,
                source,
                config_file_name,
                span_index,
                use_color,
            )?;
        }
    }

    Ok(())
}

/// Draws the header + nested `-->` blocks for an error that has a rule chain.
///
/// Each rule in the chain is indented 2 more spaces than the previous, starting at 2 spaces.
/// Ancestor blocks show only the header and source line plus a `>` connector marker; only the
/// innermost (applied) block shows the rule body with the violated entry underlined (`^`).
#[allow(clippy::too_many_arguments)]
fn print_chain_blocks<E: Reportable>(
    writer: &mut StandardStream,
    error: &E,
    chain: &[String],
    notes: &HashMap<String, String>,
    source: &str,
    config_file_name: &str,
    span_index: &SpanIndex,
    use_color: bool,
) -> Result<(), CliError> {
    // --- Resolve spans for each rule in the chain ---
    // Collect (rule_name, line, byte_col, span_text, span) for rules that have a span.
    // Rules whose span cannot be resolved are skipped; indent levels are re-packed accordingly.
    struct RuleBlock {
        name: String,
        line: usize,
        byte_col: usize,
        span_text: String,
        span_start: usize,
        span_end: usize,
    }

    let blocks: Vec<RuleBlock> = chain
        .iter()
        .filter_map(|name| {
            let span = span_index.lookup_with_ancestors(&format!("rules.{name}"))?;
            let (line, col) = byte_offset_to_line_col(source, span.start);
            let span_text = source[span.start..span.end.min(source.len())].to_string();
            Some(RuleBlock {
                name: name.clone(),
                line,
                byte_col: col,
                span_text,
                span_start: span.start,
                span_end: span.end,
            })
        })
        .collect();

    // base_indent is used for the `= hint` lines at the bottom (same as the outermost block).
    let base_indent = "  ";

    // --- Print header line: `error[CODE]: headline` ---
    {
        let mut lock = writer.lock();
        if use_color {
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                .map_err(color_err)?;
            write!(lock, "error").map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
            lock.set_color(ColorSpec::new().set_bold(true))
                .map_err(color_err)?;
            write!(lock, "[{}]", error.code()).map_err(CliError::Io)?;
            writeln!(lock, ": {}", error.headline()).map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
        } else {
            writeln!(lock, "error[{}]: {}", error.code(), error.headline())
                .map_err(CliError::Io)?;
        }
    }

    // --- Print `file: <path>` or `dir: <path>/` line ---
    if let Some((full_path, leaf, is_dir)) = error.subject() {
        let mut lock = writer.lock();
        let prefix = &full_path[..full_path.len() - leaf.len()];
        let label = if is_dir { "dir" } else { "file" };
        if use_color {
            write!(lock, "{label}: {prefix}").map_err(CliError::Io)?;
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))
                .map_err(color_err)?;
            write!(lock, "{leaf}").map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
            if is_dir {
                writeln!(lock, "/").map_err(CliError::Io)?;
            } else {
                writeln!(lock).map_err(CliError::Io)?;
            }
        } else if is_dir {
            writeln!(lock, "dir: {full_path}/").map_err(CliError::Io)?;
        } else {
            writeln!(lock, "file: {full_path}").map_err(CliError::Io)?;
        }
    }

    // --- Print each rule block with nested indentation ---
    // indent for block i (0-based) = 2 + i*2 spaces.
    let last_idx = blocks.len().saturating_sub(1);

    // Compute a single gutter width `w` shared by all blocks of this error so that the gutter pipes
    // (`|`) and chain markers (`-->` / `>`) line up in one column across the whole diagnostic. The
    // width is the digit count of the largest line number rendered anywhere in this error.
    let w = {
        let mut max_line = 0usize;
        for (i, block) in blocks.iter().enumerate() {
            let cand = if i == last_idx {
                let body_end = span_index
                    .lookup_rule_body_span(&block.name)
                    .map(|s| s.end)
                    .unwrap_or(block.span_end);
                let body_text = &source[block.span_start..body_end.min(source.len())];
                let line_count = body_text.lines().count();
                if line_count == 0 {
                    block.line
                } else {
                    block.line + line_count.saturating_sub(1)
                }
            } else {
                block.line
            };
            max_line = max_line.max(cand);
        }
        max_line.max(1).to_string().len()
    };
    let blank_field = " ".repeat(w);

    for (i, block) in blocks.iter().enumerate() {
        let is_last = i == last_idx;
        let indent = " ".repeat(2 + i * 2);

        let mut lock = writer.lock();

        // `{indent}{dashes}> rule.{name} ({config_file_name}:{line}:{col})`
        // To align the header's `>` column with the body lines' `|` column (indent width + w + 1),
        // emit w+1 `-` characters right after the indent and end with `>`.
        if use_color {
            write!(lock, "{indent}").map_err(CliError::Io)?;
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
                .map_err(color_err)?;
            write!(lock, "{}> ", "-".repeat(w + 1)).map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))
                .map_err(color_err)?;
            write!(lock, "rule.{}", block.name).map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
            write!(lock, " ").map_err(CliError::Io)?;
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))
                .map_err(color_err)?;
            writeln!(
                lock,
                "({config_file_name}:{}:{})",
                block.line, block.byte_col
            )
            .map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
        } else {
            writeln!(
                lock,
                "{indent}{}> rule.{} ({config_file_name}:{}:{})",
                "-".repeat(w + 1),
                block.name,
                block.line,
                block.byte_col
            )
            .map_err(CliError::Io)?;
        }

        if is_last {
            // Final block: print the rule definition line plus its body, underlining the violated
            // entry's line with `^`.
            let body_end = span_index
                .lookup_rule_body_span(&block.name)
                .map(|s| s.end)
                .unwrap_or(block.span_end);
            let body_text = &source[block.span_start..body_end.min(source.len())];

            // Resolve the entry span to determine the underline target.
            let entry_span = error
                .entry_path()
                .and_then(|p| span_index.lookup_with_ancestors(p));

            let mut pos = block.span_start;
            let mut first_line = true;
            let mut lineno = block.line;
            for line in body_text.lines() {
                let line_start = pos;
                let line_end = pos + line.len();

                // Decide whether to underline this line.
                // Underline only the line whose start contains the entry span start (marked_yaml's
                // span.end extends to the next line's head, so basing the decision on the start
                // avoids false positives).
                let should_underline = if let Some(es) = entry_span {
                    es.start >= line_start && es.start < line_end
                } else {
                    // Without an entry span, underline only the first (definition) line.
                    first_line
                };

                // Emit line numbers only for the rule definition line (first_line) and the error
                // line (should_underline). Other context lines are padded with blank_field and
                // show no line number.
                let show_lineno = first_line || should_underline;

                if use_color {
                    if show_lineno {
                        if should_underline {
                            // Error line: emphasize the line number in red and bold.
                            lock.set_color(
                                ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true),
                            )
                            .map_err(color_err)?;
                        } else {
                            // Rule definition line: render with the usual dimmed style.
                            lock.set_color(ColorSpec::new().set_dimmed(true))
                                .map_err(color_err)?;
                        }
                        write!(lock, "{indent}{lineno:>width$} ", width = w)
                            .map_err(CliError::Io)?;
                        lock.reset().map_err(color_err)?;
                    } else {
                        // Context line: emit no line number and pad with blank_field.
                        write!(lock, "{indent}{blank_field} ").map_err(CliError::Io)?;
                    }
                    lock.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
                        .map_err(color_err)?;
                    write!(lock, "| ").map_err(CliError::Io)?;
                    lock.reset().map_err(color_err)?;
                } else if show_lineno {
                    write!(lock, "{indent}{lineno:>width$} | ", width = w).map_err(CliError::Io)?;
                } else {
                    write!(lock, "{indent}{blank_field} | ").map_err(CliError::Io)?;
                }
                writeln!(lock, "{line}").map_err(CliError::Io)?;

                if should_underline {
                    let leading = line.chars().take_while(|c| c.is_whitespace()).count();
                    let content = line.chars().count() - leading;
                    let uline: String = std::iter::repeat_n(' ', leading)
                        .chain(std::iter::repeat_n('^', content.max(1)))
                        .collect();
                    if use_color {
                        write!(lock, "{indent}{blank_field} ").map_err(CliError::Io)?;
                        lock.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
                            .map_err(color_err)?;
                        write!(lock, "| ").map_err(CliError::Io)?;
                        lock.reset().map_err(color_err)?;
                        lock.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
                            .map_err(color_err)?;
                        writeln!(lock, "{uline}").map_err(CliError::Io)?;
                        lock.reset().map_err(color_err)?;
                    } else {
                        writeln!(lock, "{indent}{blank_field} | {uline}").map_err(CliError::Io)?;
                    }
                }

                pos += line.len() + 1;
                lineno += 1;
                first_line = false;
            }
        } else {
            // Ancestor block: print the definition line plus a `>` connector marker.
            if use_color {
                lock.set_color(ColorSpec::new().set_dimmed(true))
                    .map_err(color_err)?;
                write!(lock, "{indent}{:>width$} ", block.line, width = w).map_err(CliError::Io)?;
                lock.reset().map_err(color_err)?;
                lock.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
                    .map_err(color_err)?;
                write!(lock, "| ").map_err(CliError::Io)?;
                lock.reset().map_err(color_err)?;
            } else {
                write!(lock, "{indent}{:>width$} | ", block.line, width = w)
                    .map_err(CliError::Io)?;
            }
            writeln!(lock, "{}", block.span_text).map_err(CliError::Io)?;

            if use_color {
                lock.set_color(ColorSpec::new().set_dimmed(true))
                    .map_err(color_err)?;
                writeln!(lock, "{indent}{blank_field} >").map_err(CliError::Io)?;
                lock.reset().map_err(color_err)?;
            } else {
                writeln!(lock, "{indent}{blank_field} >").map_err(CliError::Io)?;
            }
        }
    }

    // --- Print fix hint lines at base_indent ---
    let mut lock = writer.lock();
    if let Some(rule) = error.applied_rule()
        && let Some(note) = notes.get(rule)
    {
        if use_color {
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
                .map_err(color_err)?;
            write!(lock, "{base_indent}= ").map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
            writeln!(lock, "rule '{rule}': {note}").map_err(CliError::Io)?;
        } else {
            writeln!(lock, "{base_indent}= rule '{rule}': {note}").map_err(CliError::Io)?;
        }
    }
    if let Some(hint) = error.fix_hint() {
        if use_color {
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
                .map_err(color_err)?;
            write!(lock, "{base_indent}= ").map_err(CliError::Io)?;
            lock.reset().map_err(color_err)?;
            writeln!(lock, "{hint}").map_err(CliError::Io)?;
        } else {
            writeln!(lock, "{base_indent}= {hint}").map_err(CliError::Io)?;
        }
    }
    // Blank line between diagnostics.
    writeln!(lock).map_err(CliError::Io)?;

    Ok(())
}

/// Converts a byte offset `start` in `source` to a 1-based (line, col) pair.
///
/// Both line and column are byte-based: col is the number of bytes from the start of the line plus
/// 1. Rule definition spans start at the beginning of their line, so col is typically 1.
fn byte_offset_to_line_col(source: &str, start: usize) -> (usize, usize) {
    let start = start.min(source.len());
    let before = &source[..start];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = start - before.rfind('\n').map(|i| i + 1).unwrap_or(0) + 1;
    (line, col)
}

/// Returns true when ANSI color should be embedded in message strings.
///
/// Color is used when stderr is a tty and NO_COLOR is not set.
fn should_use_color() -> bool {
    use std::io::IsTerminal;
    std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
}

/// Applies bold+cyan ANSI escape codes to `text` when `use_color` is true.
fn highlight(text: &str, use_color: bool) -> String {
    if use_color {
        format!("\x1b[1;36m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

/// Computes the gutter width (number of digits of the largest line number in labels), matching
/// codespan's rendering so that `= note` lines align. Returns 0 when no labels are present.
fn gutter_width(source: &str, labels: &[Label<()>]) -> usize {
    if labels.is_empty() {
        return 0;
    }
    let max_byte = labels.iter().map(|l| l.range.end).max().unwrap_or(0);
    let line_num = source[..max_byte.min(source.len())]
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        + 1;
    line_num.to_string().len()
}

/// Converts a color/IO-related error into a `CliError::Io`.
fn color_err(e: impl std::error::Error + Send + Sync + 'static) -> CliError {
    CliError::Io(std::io::Error::other(e))
}

/// Builds a codespan `Diagnostic` from a reportable error (without notes — those are rendered
/// separately after emit). Returns the diagnostic and the gutter width for note alignment.
fn to_diagnostic<E: Reportable>(
    error: &E,
    span_index: &SpanIndex,
    use_color: bool,
    source: &str,
) -> (Diagnostic<()>, usize) {
    // Build the message: headline + optional file: line (no rule: line).
    let mut message = error.headline();
    if let Some((full_path, leaf, is_dir)) = error.subject() {
        let prefix = full_path[..full_path.len() - leaf.len()].to_string();
        let highlighted_leaf = highlight(&leaf, use_color);
        if is_dir {
            message.push_str(&format!("\ndir: {prefix}{highlighted_leaf}/"));
        } else {
            message.push_str(&format!("\nfile: {prefix}{highlighted_leaf}"));
        }
    }

    // Attach source labels for all context paths.
    let paths = error.context_paths();
    let mut labels: Vec<Label<()>> = Vec::new();
    if !paths.is_empty() {
        let last_idx = paths.len() - 1;
        for (i, path) in paths.iter().enumerate() {
            if let Some(span) = span_index.lookup_with_ancestors(path) {
                let range = span.start..span.end;
                let label = if i == last_idx {
                    Label::primary((), range)
                } else {
                    Label::secondary((), range)
                };
                labels.push(label);
            }
        }
    }

    let width = gutter_width(source, &labels);

    let diag = Diagnostic::error()
        .with_code(error.code())
        .with_message(message)
        .with_labels(labels);

    (diag, width)
}
