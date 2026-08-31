use paperlint::{
    config::Level,
    latex::span::{SourceFile, Span},
    lint::diagnostic::Diagnostic,
    output::human::{ColorMode, HumanRenderError, render},
    rule_id::RuleId,
};
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthStr;

fn diagnostic(
    rule: RuleId,
    severity: Level,
    message: &str,
    file: &Path,
    text: &str,
    start: usize,
    end: usize,
) -> Diagnostic {
    let (line, column) = line_column(text, start);
    Diagnostic {
        rule,
        severity,
        message: message.to_string(),
        span: Span {
            file: file.to_path_buf(),
            start,
            end,
            line,
            column,
        },
    }
}

fn line_column(text: &str, byte: usize) -> (usize, usize) {
    let prefix = &text[..byte];
    let line = prefix.matches('\n').count() + prefix.matches('\r').count()
        - prefix.matches("\r\n").count()
        + 1;
    let line_start = prefix.rfind(['\n', '\r']).map_or(0, |index| index + 1);
    let column = text[line_start..byte].chars().count() + 1;
    (line, column)
}

#[test]
fn renders_compact_source_location_and_summary() {
    let path = PathBuf::from("/paper/chapters/introduction.tex");
    let text = "第一行。\n本文对该问题进行了详细分析。\n";
    let start = text.find("本文").unwrap();
    let end = start + "本文对该问题进行了详细分析。".len();
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.to_string(),
        }],
        Path::new("/paper"),
        &[diagnostic(
            RuleId::Style001,
            Level::Warning,
            "sentence is too long",
            &path,
            text,
            start,
            end,
        )],
        ColorMode::Never,
    )
    .unwrap();

    assert_eq!(
        output,
        "warning[STYLE001] chapters/introduction.tex:2:1: sentence is too long\n  本文对该问题进行了详细分析。\n  ^^^^^^^^^^^^^^^^^^^^^^^^…\n\nFound 1 problem: 0 errors, 1 warning\n"
    );
    assert!(!output.contains(['│', '─', '╭', '╯']));
    assert!(!output.contains("\x1b["));
}

#[test]
fn multiple_diagnostics_use_one_blank_separator_and_summary() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "RAG and BERT\n";
    let diagnostics = [
        diagnostic(RuleId::Acr001, Level::Error, "first", &path, text, 0, 3),
        diagnostic(RuleId::Acr002, Level::Warning, "second", &path, text, 8, 12),
    ];

    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.to_string(),
        }],
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .unwrap();

    assert_eq!(
        output,
        "error[ACR001] main.tex:1:1: first\n  RAG and BERT\n  ^^^\nwarning[ACR002] main.tex:1:9: second\n  RAG and BERT\n          ^^^^\n\nFound 2 problems: 1 error, 1 warning\n"
    );
}

#[test]
fn renders_no_problems_summary() {
    assert_eq!(
        render(&[], Path::new("/paper"), &[], ColorMode::Never).unwrap(),
        "No problems found.\n"
    );
}

#[test]
fn missing_source_returns_error_without_reading_disk() {
    let missing = PathBuf::from("/paper/not-loaded.tex");
    let diagnostics = [Diagnostic {
        rule: RuleId::Style001,
        severity: Level::Warning,
        message: "missing".to_string(),
        span: Span {
            file: missing.clone(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
        },
    }];

    let error = render(&[], Path::new("/paper"), &diagnostics, ColorMode::Never).unwrap_err();
    assert!(matches!(error, HumanRenderError::MissingSource(path) if path == missing));
}

#[test]
fn long_header_message_is_cropped_to_a_compact_width() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "TARGET\n";
    let message = "a very long diagnostic explanation ".repeat(8);
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.to_string(),
        }],
        Path::new("/paper"),
        &[diagnostic(
            RuleId::Term001,
            Level::Warning,
            &message,
            &path,
            text,
            0,
            6,
        )],
        ColorMode::Never,
    )
    .unwrap();

    let header = output.lines().next().unwrap();
    assert!(header.ends_with('…'), "{output}");
    assert!(UnicodeWidthStr::width(header) <= 100, "{output}");
}

#[test]
fn long_ascii_line_is_cropped_around_span() {
    let path = PathBuf::from("/paper/main.tex");
    let prefix = "before ".repeat(24);
    let suffix = " after".repeat(24);
    let text = format!("{prefix}TARGET{suffix}\n");
    let start = text.find("TARGET").unwrap();
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.clone(),
        }],
        Path::new("/paper"),
        &[diagnostic(
            RuleId::Term001,
            Level::Warning,
            "long line",
            &path,
            &text,
            start,
            start + 6,
        )],
        ColorMode::Never,
    )
    .unwrap();

    let lines: Vec<_> = output.lines().collect();
    assert!(lines[1].starts_with("  …"), "{output}");
    assert!(lines[1].ends_with('…'), "{output}");
    assert!(lines[1].contains("TARGET"), "{output}");
    assert!(!output.contains(&prefix));
    assert!(
        lines
            .iter()
            .all(|line| UnicodeWidthStr::width(*line) <= 100)
    );
}

#[test]
fn long_cjk_line_is_cropped_by_display_width_and_marker_aligns() {
    let path = PathBuf::from("/paper/main.tex");
    let text = format!("{}BERT{}\n", "前文".repeat(30), "后文".repeat(30));
    let start = text.find("BERT").unwrap();
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.clone(),
        }],
        Path::new("/paper"),
        &[diagnostic(
            RuleId::Acr001,
            Level::Warning,
            "acronym",
            &path,
            &text,
            start,
            start + 4,
        )],
        ColorMode::Never,
    )
    .unwrap();

    let lines: Vec<_> = output.lines().collect();
    assert!(lines[1].contains("BERT"), "{output}");
    assert_eq!(
        UnicodeWidthStr::width(&lines[1][..lines[1].find("BERT").unwrap()]),
        UnicodeWidthStr::width(&lines[2][..lines[2].find('^').unwrap()]),
        "{output}"
    );
    assert!(UnicodeWidthStr::width(lines[1]) <= 82, "{output}");
}

#[test]
fn tab_and_emoji_grapheme_keep_marker_alignment() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "prefix\t👩‍🔬BERT suffix\n";
    let start = text.find("BERT").unwrap();
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.to_string(),
        }],
        Path::new("/paper"),
        &[diagnostic(
            RuleId::Acr001,
            Level::Warning,
            "acronym",
            &path,
            text,
            start,
            start + 4,
        )],
        ColorMode::Never,
    )
    .unwrap();

    let lines: Vec<_> = output.lines().collect();
    assert!(lines[1].contains("prefix  👩‍🔬BERT suffix"), "{output}");
    assert_eq!(
        UnicodeWidthStr::width(&lines[1][..lines[1].find("BERT").unwrap()]),
        UnicodeWidthStr::width(&lines[2][..lines[2].find('^').unwrap()]),
        "{output}"
    );
}

#[test]
fn crlf_uses_derived_location_and_never_emits_carriage_return() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "first\r\nsecond BERT\r\n";
    let start = text.find("BERT").unwrap();
    let mut stale = diagnostic(
        RuleId::Acr001,
        Level::Warning,
        "acronym",
        &path,
        text,
        start,
        start + 4,
    );
    stale.span.line = 99;
    stale.span.column = 99;
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.to_string(),
        }],
        Path::new("/paper"),
        &[stale],
        ColorMode::Never,
    )
    .unwrap();

    assert!(
        output.starts_with("warning[ACR001] main.tex:2:8:"),
        "{output}"
    );
    assert!(output.contains("second BERT"));
    assert!(!output.contains('\r'));
}

#[test]
fn multiline_span_marks_first_line_as_continued() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "start TARGET\ncontinues here\n";
    let start = text.find("TARGET").unwrap();
    let end = text.find("here").unwrap() + 4;
    let output = render(
        &[SourceFile {
            path: path.clone(),
            text: text.to_string(),
        }],
        Path::new("/paper"),
        &[diagnostic(
            RuleId::Style001,
            Level::Warning,
            "multiline",
            &path,
            text,
            start,
            end,
        )],
        ColorMode::Never,
    )
    .unwrap();

    assert!(
        output.contains("  start TARGET\n        ^^^^^^…\n"),
        "{output}"
    );
    assert!(!output.contains("continues here"));
}

#[test]
fn nested_path_is_relative_and_source_path_is_unchanged() {
    let root = PathBuf::from("/paper");
    let path = root.join("chapters/nested/results.tex");
    let text = "result\n";
    let sources = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];
    let output = render(
        &sources,
        &root,
        &[diagnostic(
            RuleId::Term001,
            Level::Warning,
            "term",
            &path,
            text,
            0,
            6,
        )],
        ColorMode::Never,
    )
    .unwrap();

    assert!(output.contains("chapters/nested/results.tex:1:1:"));
    assert_eq!(sources[0].path, path);
}

#[test]
fn explicit_color_modes_control_only_header_ansi() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "BERT\n";
    let source = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Acr001,
        Level::Warning,
        "acronym",
        &path,
        text,
        0,
        4,
    )];

    let always = render(
        &source,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Always,
    )
    .unwrap();
    let never = render(&source, Path::new("/paper"), &diagnostics, ColorMode::Never).unwrap();

    assert!(always.contains("\x1b[33mwarning[ACR001]\x1b[0m"));
    assert!(always.contains("\x1b[36mmain.tex:1:1\x1b[0m"));
    assert!(!always.contains("38;5;") && !always.contains("38;2;"));
    assert!(!never.contains("\x1b["));
}

#[test]
fn rejects_invalid_utf8_byte_range_before_rendering() {
    let path = PathBuf::from("/paper/main.tex");
    let sources = [SourceFile {
        path: path.clone(),
        text: "中文\n".to_string(),
    }];
    let diagnostics = [Diagnostic {
        rule: RuleId::Style001,
        severity: Level::Warning,
        message: "bad range".to_string(),
        span: Span {
            file: path.clone(),
            start: 1,
            end: 3,
            line: 1,
            column: 1,
        },
    }];

    let error = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        HumanRenderError::InvalidByteRange {
            path: error_path,
            start: 1,
            end: 3,
            source_len: 7,
        } if error_path == path
    ));
}
