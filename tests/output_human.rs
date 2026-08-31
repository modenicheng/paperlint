use paperlint::{
    config::Level,
    latex::span::{SourceFile, Span},
    lint::diagnostic::Diagnostic,
    output::human::{ColorMode, HumanRenderError, render},
    rule_id::RuleId,
};
use std::path::{Path, PathBuf};

fn diagnostic(
    rule: RuleId,
    severity: Level,
    message: &str,
    file: &Path,
    start: usize,
    end: usize,
) -> Diagnostic {
    Diagnostic {
        rule,
        severity,
        message: message.to_string(),
        span: Span {
            file: file.to_path_buf(),
            start,
            end,
            line: 1,
            column: 1,
        },
    }
}

#[test]
fn renders_original_chinese_source_with_rule_location_and_summary() {
    let path = PathBuf::from("/paper/chapters/introduction.tex");
    let text = "第一行。\n本文对该问题进行了详细分析。\n";
    let start = text.find("本文").expect("literal occurs");
    let end = start + "本文对该问题进行了详细分析。".len();
    let sources = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Style001,
        Level::Warning,
        "sentence is too long",
        &path,
        start,
        end,
    )];

    let output = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect("render succeeds");

    assert!(output.contains("warning[STYLE001]: sentence is too long"));
    assert!(output.contains("chapters/introduction.tex:2:"));
    assert!(output.contains("本文对该问题进行了详细分析。"));
    assert!(output.contains('│') || output.contains('─'));
    assert!(output.contains("Found 1 problem: 0 errors, 1 warning"));
    assert!(!output.contains("\x1b["));
}

#[test]
fn renders_error_and_warning_summary() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "RAG and BERT\n";
    let sources = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];
    let diagnostics = [
        diagnostic(RuleId::Acr001, Level::Error, "first", &path, 0, 3),
        diagnostic(RuleId::Acr002, Level::Warning, "second", &path, 8, 12),
    ];

    let output = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect("render succeeds");

    assert!(output.contains("error[ACR001]: first"));
    assert!(output.contains("warning[ACR002]: second"));
    assert!(output.contains("Found 2 problems: 1 error, 1 warning"));
}

#[test]
fn renders_no_problems_summary() {
    let output =
        render(&[], Path::new("/paper"), &[], ColorMode::Never).expect("empty render succeeds");

    assert_eq!(output, "No problems found.\n");
}

#[test]
fn missing_source_returns_error_without_reading_disk() {
    let missing = PathBuf::from("/paper/not-loaded.tex");
    let diagnostics = [diagnostic(
        RuleId::Style001,
        Level::Warning,
        "missing",
        &missing,
        0,
        1,
    )];

    let error = render(&[], Path::new("/paper"), &diagnostics, ColorMode::Never)
        .expect_err("missing snapshot must fail");

    assert!(matches!(error, HumanRenderError::MissingSource(path) if path == missing));
}

#[test]
fn multibyte_byte_range_selects_english_text_after_chinese() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "我们采用BERT模型。\n";
    let start = text.find("BERT").expect("literal occurs");
    let diagnostics = [diagnostic(
        RuleId::Acr001,
        Level::Warning,
        "acronym",
        &path,
        start,
        start + "BERT".len(),
    )];
    let sources = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];

    let output = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect("render succeeds");

    assert!(output.contains("我们采用BERT模型。"));
    assert!(output.contains("main.tex:1:"));
}

#[test]
fn tab_before_label_renders_target_line() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "prefix\tBERT suffix\n";
    let start = text.find("BERT").expect("literal occurs");
    let sources = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Acr001,
        Level::Warning,
        "acronym",
        &path,
        start,
        start + 4,
    )];

    let output = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect("render succeeds");

    assert!(output.contains("prefix"));
    assert!(output.contains("BERT suffix"));
}

#[test]
fn crlf_byte_range_renders_second_line_without_carriage_returns() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "first\r\nsecond BERT\r\n";
    let start = text.find("BERT").expect("literal occurs");
    let sources = [SourceFile {
        path: path.clone(),
        text: text.to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Acr001,
        Level::Warning,
        "acronym",
        &path,
        start,
        start + 4,
    )];

    let output = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect("render succeeds");

    assert!(output.contains("second BERT"));
    assert!(!output.contains('\r'));
}

#[test]
fn nested_path_is_relative_and_source_path_is_unchanged() {
    let root = PathBuf::from("/paper");
    let path = root.join("chapters").join("nested").join("results.tex");
    let original_path = path.clone();
    let sources = [SourceFile {
        path: path.clone(),
        text: "result\n".to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Term001,
        Level::Warning,
        "term",
        &path,
        0,
        6,
    )];

    let output = render(&sources, &root, &diagnostics, ColorMode::Never).expect("render succeeds");

    assert!(output.contains("chapters/nested/results.tex:1:"));
    assert_eq!(sources[0].path, original_path);
}

#[test]
fn explicit_color_modes_control_ansi_output() {
    let path = PathBuf::from("/paper/main.tex");
    let sources = [SourceFile {
        path: path.clone(),
        text: "BERT\n".to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Acr001,
        Level::Warning,
        "acronym",
        &path,
        0,
        4,
    )];

    let always = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Always,
    )
    .expect("color render succeeds");
    let never = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect("plain render succeeds");
    let always_again = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Always,
    )
    .expect("second color render succeeds");

    assert!(always.contains("\x1b["));
    assert!(!always.contains("38;5;"));
    assert!(!always.contains("48;5;"));
    assert!(!always.contains("38;2;"));
    assert!(!always.contains("48;2;"));
    assert!(!never.contains("\x1b["));
    assert!(always_again.contains("\x1b["));
}

#[test]
fn deterministic_no_color_golden_for_cjk_marker_alignment() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "前缀BERT后缀。\n";
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
            start,
            start + 4,
        )],
        ColorMode::Never,
    )
    .unwrap();

    assert_eq!(
        output,
        "warning[ACR001]: acronym\n   ╭─[ main.tex:1:3 ]\n   │\n 1 │ 前缀BERT后缀。\n   │     ──┬─    \n   │       ╰───── \n───╯\nFound 1 problem: 0 errors, 1 warning\n"
    );
}

#[test]
fn deterministic_no_color_golden_for_tab_marker_alignment() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "prefix\tBERT suffix\n";
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
            start,
            start + 4,
        )],
        ColorMode::Never,
    )
    .unwrap();

    assert_eq!(
        output,
        "warning[ACR001]: acronym\n   ╭─[ main.tex:1:8 ]\n   │\n 1 │ prefix  BERT suffix\n   │         ──┬─  \n   │           ╰─── \n───╯\nFound 1 problem: 0 errors, 1 warning\n"
    );
}

#[test]
fn deterministic_no_color_golden_for_crlf_second_line() {
    let path = PathBuf::from("/paper/main.tex");
    let text = "first\r\nsecond BERT\r\n";
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
            start,
            start + 4,
        )],
        ColorMode::Never,
    )
    .unwrap();

    assert_eq!(
        output,
        "warning[ACR001]: acronym\n   ╭─[ main.tex:2:8 ]\n   │\n 2 │ second BERT\n   │        ──┬─  \n   │          ╰─── \n───╯\nFound 1 problem: 0 errors, 1 warning\n"
    );
}

#[test]
fn rejects_invalid_utf8_byte_range_before_rendering() {
    let path = PathBuf::from("/paper/main.tex");
    let sources = [SourceFile {
        path: path.clone(),
        text: "中文\n".to_string(),
    }];
    let diagnostics = [diagnostic(
        RuleId::Style001,
        Level::Warning,
        "bad range",
        &path,
        1,
        3,
    )];

    let error = render(
        &sources,
        Path::new("/paper"),
        &diagnostics,
        ColorMode::Never,
    )
    .expect_err("non-boundary byte range must fail");

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
