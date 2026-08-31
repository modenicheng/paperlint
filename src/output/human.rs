use crate::{
    config::Level, latex::span::SourceFile, lint::diagnostic::Diagnostic,
    output::path::display_path,
};
use ariadne::{CharSet, Color, Config, IndexType, Label, Report, ReportKind, sources};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    string::FromUtf8Error,
    sync::Mutex,
};
use thiserror::Error;

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
    #[error("failed to write human diagnostic")]
    Write(#[source] io::Error),
    #[error("human diagnostic output was not UTF-8")]
    Utf8(#[source] FromUtf8Error),
}

// Ariadne's auto-color integration consults concolor's process-global override.
// Serializing render calls prevents explicit modes from racing within this process.
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

    let display_paths: HashMap<&Path, String> = source_files
        .iter()
        .map(|source| (source.path.as_path(), display_path(&source.path, root)))
        .collect();
    let cached_sources: Vec<(String, String)> = source_files
        .iter()
        .map(|source| {
            (
                display_paths[source.path.as_path()].clone(),
                source.text.clone(),
            )
        })
        .collect();

    let mut output = Vec::new();
    let _color_guard = COLOR_OVERRIDE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    concolor::set(color_choice(color));
    let write_result = write_reports(
        &mut output,
        cached_sources,
        &display_paths,
        diagnostics,
        color,
    );
    concolor::set(concolor::ColorChoice::Auto);
    write_result?;

    let output = String::from_utf8(output).map_err(HumanRenderError::Utf8)?;
    let mut output = normalize_ansi_palette(&output).replace('\r', "");
    output.push_str(&summary(diagnostics));
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
    output: &mut Vec<u8>,
    cached_sources: Vec<(String, String)>,
    display_paths: &HashMap<&Path, String>,
    diagnostics: &[Diagnostic],
    color: ColorMode,
) -> Result<(), HumanRenderError> {
    let mut cache = sources(cached_sources);
    let config = Config::default()
        .with_index_type(IndexType::Byte)
        .with_char_set(CharSet::Unicode)
        .with_tab_width(4)
        .with_color(color != ColorMode::Never);

    for diagnostic in diagnostics {
        let Some((kind_name, kind_color)) = report_kind(diagnostic) else {
            continue;
        };
        let source_id = display_paths[diagnostic.span.file.as_path()].clone();
        let span = diagnostic.span.start..diagnostic.span.end;
        Report::build(
            ReportKind::Custom(&kind_name, kind_color),
            (source_id.clone(), span.clone()),
        )
        .with_config(config)
        .with_message(&diagnostic.message)
        .with_label(
            Label::new((source_id, span))
                .with_color(kind_color)
                .with_message(""),
        )
        .finish()
        .write_for_stdout(&mut cache, &mut *output)
        .map_err(HumanRenderError::Write)?;
    }
    Ok(())
}

fn normalize_ansi_palette(output: &str) -> String {
    let mut normalized = String::with_capacity(output.len());
    let mut remaining = output;
    while let Some(start) = remaining.find("\x1b[") {
        normalized.push_str(&remaining[..start]);
        let escape = &remaining[start..];
        let Some(end) = escape.find('m') else {
            normalized.push_str(escape);
            return normalized;
        };
        let sequence = &escape[..=end];
        let parameters = &sequence[2..sequence.len() - 1];
        if parameters.starts_with("38;5;") || parameters.starts_with("38;2;") {
            normalized.push_str("\x1b[36m");
        } else if parameters.starts_with("48;5;") || parameters.starts_with("48;2;") {
            normalized.push_str("\x1b[46m");
        } else {
            normalized.push_str(sequence);
        }
        remaining = &escape[end + 1..];
    }
    normalized.push_str(remaining);
    normalized
}

fn report_kind(diagnostic: &Diagnostic) -> Option<(String, Color)> {
    match diagnostic.severity {
        Level::Error => Some((format!("error[{}]", diagnostic.rule), Color::Red)),
        Level::Warning => Some((format!("warning[{}]", diagnostic.rule), Color::Yellow)),
        Level::Off => None,
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
        "Found {problems} {}: {errors} {}, {warnings} {}\n",
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
