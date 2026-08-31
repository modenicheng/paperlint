use crate::{
    config::Level, latex::span::SourceFile, lint::diagnostic::Diagnostic,
    output::path::display_path,
};
use std::{
    collections::HashMap,
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const TAB_WIDTH: usize = 4;
const MAX_SNIPPET_WIDTH: usize = 80;
const MAX_HEADER_WIDTH: usize = 100;
const MAX_MARKER_WIDTH: usize = 24;
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: usize = 1;
const ERROR_COLOR: &str = "\x1b[31m";
const WARNING_COLOR: &str = "\x1b[33m";
const LOCATION_COLOR: &str = "\x1b[36m";
const RESET_COLOR: &str = "\x1b[0m";

/// Controls ANSI styling in human-readable diagnostic output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Error)]
pub enum HumanRenderError {
    #[error("diagnostic source was not loaded: {0}")]
    MissingSource(PathBuf),
    #[error(
        "invalid byte range {start}..{end} for diagnostic source {path} (source length {source_len})"
    )]
    InvalidByteRange {
        path: PathBuf,
        start: usize,
        end: usize,
        source_len: usize,
    },
}

// Concolor's auto-color integration consults a process-global override. Serializing
// render calls prevents explicit modes from racing within this process.
static COLOR_OVERRIDE: Mutex<()> = Mutex::new(());

pub fn render(
    source_files: &[SourceFile],
    root: &Path,
    diagnostics: &[Diagnostic],
    color: ColorMode,
) -> Result<String, HumanRenderError> {
    let source_by_path: HashMap<&Path, &SourceFile> = source_files
        .iter()
        .map(|source| (source.path.as_path(), source))
        .collect();

    for diagnostic in diagnostics {
        let source = source_by_path
            .get(diagnostic.span.file.as_path())
            .ok_or_else(|| HumanRenderError::MissingSource(diagnostic.span.file.clone()))?;
        validate_range(source, diagnostic)?;
    }

    let _color_guard = COLOR_OVERRIDE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    concolor::set(color_choice(color));
    let use_color = concolor::get(concolor::Stream::Stdout).ansi_color();
    let output = write_reports(&source_by_path, root, diagnostics, use_color);
    concolor::set(concolor::ColorChoice::Auto);
    Ok(output)
}

fn validate_range(source: &SourceFile, diagnostic: &Diagnostic) -> Result<(), HumanRenderError> {
    let start = diagnostic.span.start;
    let end = diagnostic.span.end;
    if start > end
        || end > source.text.len()
        || !source.text.is_char_boundary(start)
        || !source.text.is_char_boundary(end)
    {
        return Err(HumanRenderError::InvalidByteRange {
            path: diagnostic.span.file.clone(),
            start,
            end,
            source_len: source.text.len(),
        });
    }
    Ok(())
}

fn write_reports(
    source_by_path: &HashMap<&Path, &SourceFile>,
    root: &Path,
    diagnostics: &[Diagnostic],
    use_color: bool,
) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        let Some(severity) = severity_label(diagnostic.severity) else {
            continue;
        };
        let source = source_by_path[diagnostic.span.file.as_path()];
        let snippet = source_snippet(source, diagnostic);
        let (line, column) = line_column(&source.text, diagnostic.span.start);

        let severity_plain = format!("{severity}[{}]", diagnostic.rule);
        let location_plain = format!(
            "{}:{line}:{column}",
            display_path(&diagnostic.span.file, root)
        );
        let fixed_width = UnicodeWidthStr::width(severity_plain.as_str())
            + UnicodeWidthStr::width(location_plain.as_str())
            + 3;
        let message_width = MAX_HEADER_WIDTH.saturating_sub(fixed_width);
        let message = truncate_text(&diagnostic.message, message_width);
        let severity = paint(
            &severity_plain,
            severity_color(diagnostic.severity),
            use_color,
        );
        let location = paint(&location_plain, LOCATION_COLOR, use_color);
        let _ = writeln!(output, "{severity} {location}: {message}");
        let _ = writeln!(output, "  {}", snippet.text);
        let _ = writeln!(
            output,
            "  {}{}",
            " ".repeat(snippet.marker_start),
            marker(snippet.marker_width, snippet.continues)
        );
    }
    output.push_str(&summary(diagnostics));
    output
}

#[derive(Debug, PartialEq, Eq)]
struct Snippet {
    text: String,
    marker_start: usize,
    marker_width: usize,
    continues: bool,
}

fn source_snippet(source: &SourceFile, diagnostic: &Diagnostic) -> Snippet {
    let line = source_line(&source.text, diagnostic.span.start);
    let start_in_line = diagnostic.span.start.saturating_sub(line.start);
    let end_in_line = diagnostic.span.end.min(line.end).saturating_sub(line.start);
    let expanded = expand_tabs(&source.text[line.start..line.end], TAB_WIDTH);
    let span_start = expanded.start_column(start_in_line);
    let span_end = expanded.end_column(end_in_line);
    let marker_width = span_end.saturating_sub(span_start).max(1);
    let continues = diagnostic.span.end > line.end;
    crop_snippet(&expanded.text, span_start, marker_width, continues)
}

#[derive(Debug)]
struct SourceLine {
    start: usize,
    end: usize,
}

fn source_line(text: &str, byte: usize) -> SourceLine {
    let bytes = text.as_bytes();
    let mut start = byte;
    while start > 0 && !matches!(bytes[start - 1], b'\n' | b'\r') {
        start -= 1;
    }
    let mut end = byte;
    while end < bytes.len() && !matches!(bytes[end], b'\n' | b'\r') {
        end += 1;
    }
    SourceLine { start, end }
}

fn line_column(text: &str, byte: usize) -> (usize, usize) {
    let line = source_line(text, byte);
    let line_number = text.as_bytes()[..line.start]
        .iter()
        .filter(|&&value| value == b'\n' || value == b'\r')
        .count()
        + 1
        - text.as_bytes()[..line.start]
            .windows(2)
            .filter(|pair| *pair == b"\r\n")
            .count();
    let column = text[line.start..byte].chars().count() + 1;
    (line_number, column)
}

#[derive(Debug)]
struct ExpandedSegment {
    original: std::ops::Range<usize>,
    columns: std::ops::Range<usize>,
}

#[derive(Debug)]
struct ExpandedLine {
    text: String,
    segments: Vec<ExpandedSegment>,
    width: usize,
}

impl ExpandedLine {
    fn start_column(&self, original_byte: usize) -> usize {
        self.segments
            .iter()
            .find_map(|segment| {
                (segment.original.start <= original_byte && original_byte < segment.original.end)
                    .then_some(segment.columns.start)
            })
            .unwrap_or(self.width)
    }

    fn end_column(&self, original_byte: usize) -> usize {
        if original_byte == 0 {
            return 0;
        }
        self.segments
            .iter()
            .find_map(|segment| {
                (segment.original.start < original_byte && original_byte <= segment.original.end)
                    .then_some(segment.columns.end)
            })
            .unwrap_or(self.width)
    }
}

fn expand_tabs(line: &str, tab_width: usize) -> ExpandedLine {
    let mut text = String::new();
    let mut segments = Vec::with_capacity(line.graphemes(true).count());
    let mut column = 0;
    for (byte, grapheme) in line.grapheme_indices(true) {
        let column_start = column;
        if grapheme == "\t" {
            let spaces = tab_width - column % tab_width;
            text.push_str(&" ".repeat(spaces));
            column += spaces;
        } else if grapheme.chars().any(char::is_control) {
            text.push(' ');
            column += 1;
        } else {
            text.push_str(grapheme);
            column += UnicodeWidthStr::width(grapheme);
        }
        segments.push(ExpandedSegment {
            original: byte..byte + grapheme.len(),
            columns: column_start..column,
        });
    }
    ExpandedLine {
        text,
        segments,
        width: column,
    }
}

fn crop_snippet(text: &str, span_start: usize, span_width: usize, continues: bool) -> Snippet {
    let total_width = UnicodeWidthStr::width(text);
    if total_width <= MAX_SNIPPET_WIDTH {
        let visible_width = span_width
            .min(total_width.saturating_sub(span_start))
            .max(1);
        return Snippet {
            text: text.to_string(),
            marker_start: span_start,
            marker_width: visible_width.min(MAX_MARKER_WIDTH),
            continues: continues || visible_width > MAX_MARKER_WIDTH,
        };
    }

    let effective_span_width = span_width.min(MAX_SNIPPET_WIDTH.saturating_sub(2)).max(1);
    let context_width = MAX_SNIPPET_WIDTH.saturating_sub(effective_span_width);
    let desired_start = span_start.saturating_sub(context_width / 2);
    let max_start = total_width.saturating_sub(MAX_SNIPPET_WIDTH);
    let window_start = desired_start.min(max_start);
    let window_end = window_start + MAX_SNIPPET_WIDTH;
    let left_clipped = window_start > 0;
    let right_clipped = window_end < total_width;
    let content_start = window_start + usize::from(left_clipped) * ELLIPSIS_WIDTH;
    let content_end = window_end.saturating_sub(usize::from(right_clipped) * ELLIPSIS_WIDTH);

    let slice = slice_columns(text, content_start, content_end);
    let mut cropped = String::new();
    if left_clipped {
        cropped.push_str(ELLIPSIS);
    }
    cropped.push_str(&slice.text);
    if right_clipped {
        cropped.push_str(ELLIPSIS);
    }

    let visible_span_start = span_start.max(slice.start);
    let visible_span_end = (span_start + span_width).min(slice.end);
    let marker_start =
        usize::from(left_clipped) * ELLIPSIS_WIDTH + visible_span_start.saturating_sub(slice.start);
    let visible_width = visible_span_end.saturating_sub(visible_span_start).max(1);
    Snippet {
        text: cropped,
        marker_start,
        marker_width: visible_width.min(MAX_MARKER_WIDTH),
        continues: continues
            || span_start < slice.start
            || span_start + span_width > slice.end
            || visible_width > MAX_MARKER_WIDTH,
    }
}

#[derive(Debug)]
struct ColumnSlice {
    text: String,
    start: usize,
    end: usize,
}

fn slice_columns(text: &str, start: usize, end: usize) -> ColumnSlice {
    let mut output = String::new();
    let mut column = 0;
    let mut actual_start = None;
    let mut actual_end = start;
    for grapheme in text.graphemes(true) {
        let width = UnicodeWidthStr::width(grapheme);
        let next = column + width;
        if column >= start && next <= end {
            actual_start.get_or_insert(column);
            actual_end = next;
            output.push_str(grapheme);
        }
        column = next;
        if column >= end {
            break;
        }
    }
    ColumnSlice {
        text: output,
        start: actual_start.unwrap_or(start),
        end: actual_end,
    }
}

fn truncate_text(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= ELLIPSIS_WIDTH {
        return ELLIPSIS.to_string();
    }
    let slice = slice_columns(text, 0, max_width - ELLIPSIS_WIDTH);
    format!("{}{ELLIPSIS}", slice.text)
}

fn marker(width: usize, continues: bool) -> String {
    if continues {
        format!("{}…", "^".repeat(width))
    } else {
        "^".repeat(width)
    }
}

fn paint(text: &str, color: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{text}{RESET_COLOR}")
    } else {
        text.to_string()
    }
}

fn severity_label(level: Level) -> Option<&'static str> {
    match level {
        Level::Error => Some("error"),
        Level::Warning => Some("warning"),
        Level::Off => None,
    }
}

fn severity_color(level: Level) -> &'static str {
    match level {
        Level::Error => ERROR_COLOR,
        Level::Warning => WARNING_COLOR,
        Level::Off => RESET_COLOR,
    }
}

fn summary(diagnostics: &[Diagnostic]) -> String {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Level::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Level::Warning)
        .count();
    let problems = errors + warnings;

    if problems == 0 {
        return "No problems found.\n".to_string();
    }

    format!(
        "\nFound {problems} {}: {errors} {}, {warnings} {}\n",
        plural(problems, "problem", "problems"),
        plural(errors, "error", "errors"),
        plural(warnings, "warning", "warnings"),
    )
}

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn color_choice(mode: ColorMode) -> concolor::ColorChoice {
    match mode {
        ColorMode::Auto => concolor::ColorChoice::Auto,
        ColorMode::Always => concolor::ColorChoice::AlwaysAnsi,
        ColorMode::Never => concolor::ColorChoice::Never,
    }
}
