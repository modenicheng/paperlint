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
    config.rules.case001.level = Level::Off;
    config.rules.punc002.level = Level::Off;
    config
}

fn case_config() -> PaperlintConfig {
    let mut config = base_config();
    config.rules.case001.level = Level::Warning;
    config
}

fn acr_config() -> PaperlintConfig {
    let mut config = base_config();
    config.rules.acr001.level = Level::Error;
    config
}

fn lint(source: &str, configure: impl FnOnce(&mut PaperlintConfig)) -> (Document, Vec<Diagnostic>) {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, source);
    let mut config = base_config();
    configure(&mut config);
    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    (document, diagnostics)
}

fn span_text<'a>(document: &'a Document, diagnostic: &Diagnostic) -> &'a str {
    let source = document.source(&diagnostic.span.file).unwrap();
    &source.text[diagnostic.span.start..diagnostic.span.end]
}

#[test]
fn acr001_stays_silent_for_builtin_uppercase_acronyms() {
    for text in [
        "导出为PDF文件。数据以CSV格式保存。",
        "通过HTTP与HTTPS访问URL。",
        "序列由DNA编码。",
    ] {
        let (_, diagnostics) = lint(text, |config| {
            config.rules.acr001.level = Level::Error;
        });
        assert!(diagnostics.is_empty(), "{text}: {diagnostics:?}");
    }
}

#[test]
fn acr001_still_reports_unknown_acronyms_and_definitions_are_kept() {
    let (document, diagnostics) = lint(
        "GWAS用于分析。检索增强生成（Retrieval-Augmented Generation，RAG）方法。",
        |config| {
            config.rules.acr001.level = Level::Error;
        },
    );
    let messages: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages, ["acronym `GWAS` used before definition"]);
    assert_eq!(span_text(&document, &diagnostics[0]), "GWAS");
}

#[test]
fn acr001_workspace_lexicon_entry_suppresses_an_acronym() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[[lexicon.entries]]
canonical = "GWAS"
kind = "common"
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, acr_config());
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "GWAS用于分析。");
    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn case001_reports_wrong_cased_builtin_proper_nouns_and_units() {
    let (document, diagnostics) = lint(
        "代码托管在Github。模型用Pytorch训练。文档由Latex排版。频率为5Ghz。",
        |config| {
            config.rules.case001.level = Level::Warning;
        },
    );
    let seen: Vec<(&str, &str, &str)> = diagnostics
        .iter()
        .map(|diagnostic| {
            (
                span_text(&document, diagnostic),
                match diagnostic.rule {
                    RuleId::Case001 => "CASE001",
                    _ => "other",
                },
                if diagnostic.severity == Level::Warning {
                    "warning"
                } else {
                    "other"
                },
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("Github", "CASE001", "warning"),
            ("Pytorch", "CASE001", "warning"),
            ("Latex", "CASE001", "warning"),
            ("Ghz", "CASE001", "warning"),
        ]
    );
    let github = &diagnostics[0];
    assert_eq!(
        github.message, "use `GitHub` instead of `Github`",
        "{}",
        github.message
    );
}

#[test]
fn case001_stays_silent_for_unknown_and_lowercase_variants() {
    let (_, diagnostics) = lint(
        "MyModel与Wordpress表现良好。词语rag与rag无关。速度5ms与10ns达标。",
        |config| {
            config.rules.case001.level = Level::Warning;
        },
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn case001_reports_document_defined_acronym_mixed_case_but_not_lowercase() {
    let (document, diagnostics) = lint(
        "检索增强生成（Retrieval-Augmented Generation，RAG）方法。Rag与RAG变体。",
        |config| {
            config.rules.case001.level = Level::Warning;
        },
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(span_text(&document, &diagnostics[0]), "Rag");
    assert_eq!(diagnostics[0].message, "use `RAG` instead of `Rag`");
}

#[test]
fn case001_reports_github_lowercase_variant() {
    let (_, diagnostics) = lint("开源社区github提供了代码。", |config| {
        config.rules.case001.level = Level::Warning;
    });
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].message, "use `GitHub` instead of `github`");
}

#[test]
fn case001_reports_exact_source_span_inside_cjk_text() {
    let (document, diagnostics) = lint("模型部署在\\textbf{Github}平台上。", |config| {
        config.rules.case001.level = Level::Warning;
    });
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(span_text(&document, &diagnostics[0]), "Github");
    let source = document.source(&diagnostics[0].span.file).unwrap();
    assert_eq!(
        diagnostics[0].span.start,
        source.text.find("Github").unwrap()
    );
    assert_eq!(diagnostics[0].span.end, diagnostics[0].span.start + 6);
}

#[test]
fn term001_defers_case_only_replacement_to_case001() {
    let (_, diagnostics) = lint(
        "使用Github作为术语。data set仍由TERM001处理。",
        |config| {
            config.rules.term001.level = Level::Warning;
            config.rules.case001.level = Level::Warning;
        },
    );
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    let github_reports: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("GitHub"))
        .collect();
    assert_eq!(github_reports.len(), 1, "{diagnostics:?}");
    assert_eq!(github_reports[0].rule, RuleId::Case001);
    assert_eq!(github_reports[0].severity, Level::Warning);
    let dataset_reports: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("dataset"))
        .collect();
    assert_eq!(dataset_reports.len(), 1);
    assert_eq!(dataset_reports[0].rule, RuleId::Term001);
}

#[test]
fn case001_off_and_cli_disable_override() {
    let (_, diagnostics) = lint("使用Github。", |config| {
        config.rules.case001.level = Level::Off;
    });
    assert!(diagnostics.is_empty());

    let mut config = case_config();
    config = config.with_overrides(&[], &["CASE001".to_string()]);
    assert_eq!(config.rules.case001.level, Level::Off);

    let mut config = case_config();
    config.rules.case001.level = Level::Off;
    config = config.with_overrides(&["CASE001".to_string()], &[]);
    assert_eq!(config.rules.case001.level, Level::Warning);
}

#[test]
fn case001_config_loads_from_raw_toml() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.CASE001]
level = "error"
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, base_config());
    assert_eq!(config.rules.case001.level, Level::Error);
    assert_eq!(DefaultConfig::load().rules.case001.level, Level::Warning);
}
