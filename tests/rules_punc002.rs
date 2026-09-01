use paperlint::{
    config::{DefaultConfig, Level, PaperlintConfig, RawPaperlintConfig},
    latex::parser::{self, Document},
    lint::{diagnostic::Diagnostic, engine::RuleEngine, registry::RuleRegistry},
    rule_id::RuleId,
};
use std::{fs, path::Path};
use tempfile::tempdir;

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn base_config() -> PaperlintConfig {
    let mut config = DefaultConfig::load();
    config.rules.acr001.level = Level::Off;
    config.rules.acr002.level = Level::Off;
    config.rules.term001.level = Level::Off;
    config.rules.style001.level = Level::Off;
    config
}

fn lint(source: &str, configure: impl FnOnce(&mut PaperlintConfig)) -> (Document, Vec<Diagnostic>) {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, source);
    let mut config = base_config();
    configure(&mut config);
    let document = parser_parse(&main, &config);
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    (document, diagnostics)
}

// Kept in one place so tests fail with the parser error instead of unwrap noise.
fn parser_parse(main: &Path, config: &PaperlintConfig) -> Document {
    parser::parse(main.to_path_buf(), &config.latex).unwrap_or_else(|error| panic!("{error}"))
}

fn span_text<'a>(document: &'a Document, diagnostic: &Diagnostic) -> &'a str {
    let source = document.source(&diagnostic.span.file).unwrap();
    &source.text[diagnostic.span.start..diagnostic.span.end]
}

#[test]
fn punc002_reports_both_missing_space_boundaries_around_a_latin_token() {
    let (document, diagnostics) = lint("使用LLM进行推理。", |_| ());
    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule == RuleId::Punc002)
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity == Level::Warning)
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message == "missing space between CJK and Latin/number")
    );
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["用L", "M进"]);
}

#[test]
fn punc002_accepts_ascii_and_fullwidth_spaces_between_scripts() {
    let (document, diagnostics) = lint("使用 LLM 进行推理。\n使用　LLM。", |_| ());
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?} {}",
        diagnostics
            .iter()
            .map(|diagnostic| span_text(&document, diagnostic))
            .collect::<Vec<_>>()
            .join(",")
    );
}

#[test]
fn punc002_reports_digit_boundaries_inside_chinese_text() {
    let (document, diagnostics) = lint("共100个样本。", |_| ());
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["共1", "0个"]);
}

#[test]
fn punc002_never_reports_latin_digit_or_percent_pairs() {
    let (document, diagnostics) = lint("长度3cm占比50%。", |_| ());
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["度3", "m占", "比5"]);
    assert!(!spans.iter().any(|span| *span == "3c" || *span == "0%"));
}

#[test]
fn punc002_punctuation_does_not_trigger_and_does_not_mask_boundaries() {
    let (document, diagnostics) = lint("模型（LLM）不可用。\n使用LLM。", |_| ());
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(span_text(&document, &diagnostics[0]), "用L");
}

#[test]
fn punc002_default_exemptions_cover_chapter_figure_and_table_references() {
    let (document, diagnostics) = lint(
        "第3章介绍。\n如图2所示。\n见表1。\n方向1A与隔离2B。",
        |_| (),
    );
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["向1", "A与", "离2"]);
}

#[test]
fn punc002_reports_reference_boundaries_once_ignore_patterns_are_cleared() {
    let (document, diagnostics) = lint("第3章共有50页。", |config| {
        config.rules.punc002.ignore_patterns = Vec::new();
    });
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["第3", "3章", "有5", "0页"]);
}

#[test]
fn punc002_skips_boundaries_dropped_by_empty_formatting_groups() {
    // \textbf{} leaves a source gap between 用 and L, so that pair is a
    // parsing artifact; M and 推 stay source-contiguous and still report.
    let (document, diagnostics) = lint("使用\\textbf{}LLM推理。", |_| ());
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(span_text(&document, &diagnostics[0]), "M推");
}

#[test]
fn punc002_skips_allowbreak_gaps_inside_compound_words() {
    // Only the R|P boundary spans the dropped "/\\allowbreak{}" source bytes
    // and is skipped; the outer word edges 用|R and n|实 are real text and
    // still report.
    let (document, diagnostics) = lint("使用R/\\allowbreak{}Python实现。", |_| ());
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["用R", "n实"]);
}

#[test]
fn punc002_skips_both_sides_when_parser_drops_bracket_bytes_around_argument() {
    // Logical "中X英" with X from \textbf{X}: the parser drops both braces,
    // so every boundary into X spans a source gap and is skipped.
    let (document, diagnostics) = lint("中\\textbf{X}英。", |_| ());
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        diagnostics
            .iter()
            .map(|diagnostic| span_text(&document, diagnostic))
            .collect::<Vec<_>>()
    );
}

#[test]
fn punc002_reports_included_child_file_with_exact_span() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    write(&main, "\\input{child}主文件正文。");
    write(&child, "本文使用LLM推理。");
    let config = base_config();
    let document = parser_parse(&main, &config);
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].span.file, child.canonicalize().unwrap());
    assert_eq!(diagnostics[1].span.file, child.canonicalize().unwrap());
    let source = document.source(&diagnostics[0].span.file).unwrap();
    assert_eq!(&source.text[diagnostics[0].span.start..], "用LLM推理。");
    assert_eq!(span_text(&document, &diagnostics[0]), "用L");
    assert_eq!(span_text(&document, &diagnostics[1]), "M推");
}

#[test]
fn punc002_off_does_not_execute() {
    let (_, diagnostics) = lint("使用LLM进行推理。", |config| {
        config.rules.punc002.level = Level::Off;
    });
    assert!(diagnostics.is_empty());
}

#[test]
fn punc002_level_and_patterns_load_from_raw_config() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.PUNC002]
level = "error"
ignore_patterns = ["第\\d+章"]
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, base_config());
    assert_eq!(config.rules.punc002.level, Level::Error);
    assert_eq!(config.rules.punc002.ignore_patterns, [r"第\d+章"]);

    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "第3章。使用LLM。");
    let document = parser_parse(&main, &config);
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Level::Error);
    assert_eq!(span_text(&document, &diagnostics[0]), "用L");
}

#[test]
fn punc002_invalid_ignore_regex_is_ignored_without_panicking() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.PUNC002]
level = "warning"
ignore_patterns = ["([bad"]
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, base_config());

    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "第3章。使用LLM。");
    let document = parser_parse(&main, &config);
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| span_text(&document, diagnostic))
        .collect();
    assert_eq!(spans, ["第3", "3章", "用L"]);
}
