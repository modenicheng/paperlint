use paperlint::{
    config::{DefaultConfig, Level},
    latex::parser,
    lint::{engine::RuleEngine, registry::RuleRegistry},
    rule_id::RuleId,
};
use std::{fs, path::Path};
use tempfile::tempdir;

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn config_for(rule: RuleId) -> paperlint::config::PaperlintConfig {
    let mut config = DefaultConfig::load();
    config.rules.acr001.level = Level::Off;
    config.rules.acr002.level = Level::Off;
    config.rules.term001.level = Level::Off;
    config.rules.style001.level = Level::Off;
    match rule {
        RuleId::Acr001 => config.rules.acr001.level = Level::Warning,
        RuleId::Acr002 => config.rules.acr002.level = Level::Warning,
        RuleId::Term001 => config.rules.term001.level = Level::Warning,
        RuleId::Style001 => config.rules.style001.level = Level::Warning,
        _ => unreachable!(),
    }
    config
}

#[test]
fn reports_chinese_sentence_over_max_chars_at_child_source_span() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("chapter.tex");
    write(&main, "\\input{chapter}");
    let sentence = format!("{}。", "中".repeat(81));
    write(&child, &sentence);
    let config = config_for(RuleId::Style001);
    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.rule, RuleId::Style001);
    assert_eq!(diagnostic.severity, Level::Warning);
    assert_eq!(diagnostic.span.file, child.canonicalize().unwrap());
    let source = document.source(&diagnostic.span.file).unwrap();
    assert_eq!(
        &source.text[diagnostic.span.start..diagnostic.span.end],
        sentence
    );
}

#[test]
fn does_not_report_short_chinese_sentence() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, &format!("{}。", "中".repeat(80)));
    let config = config_for(RuleId::Style001);
    let document = parser::parse(main, &config.latex).unwrap();
    assert!(
        RuleEngine::new(RuleRegistry::default())
            .run(&document, &config)
            .is_empty()
    );
}

#[test]
fn english_uses_max_english_words_and_mixed_uses_effective_length() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "one two three four.\n中文中文 LLM。");
    let mut config = config_for(RuleId::Style001);
    config.rules.style001.max_english_words = 3;
    config.rules.style001.max_chars = 4;
    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics[0].message.contains("4 words; max is 3"));
    assert!(
        diagnostics[1]
            .message
            .contains("5 effective characters; max is 4")
    );
}

#[test]
fn style_span_crosses_formatting_gap_and_trims_leading_crlf_whitespace() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let sentence = format!("中\\textbf{{{}}}。", "文".repeat(5));
    write(&main, &format!("\r\n  {sentence}\r\n"));
    let mut config = config_for(RuleId::Style001);
    config.rules.style001.max_chars = 5;

    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    let source = document.source(&diagnostic.span.file).unwrap();
    assert_eq!(
        &source.text[diagnostic.span.start..diagnostic.span.end],
        sentence
    );
    assert_eq!(diagnostic.span.start, source.text.find('中').unwrap());
}

#[test]
fn acr002_threshold_two_allows_two_cross_file_occurrences() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let first = dir.path().join("first.tex");
    let second = dir.path().join("second.tex");
    write(&main, "\\input{first}\\input{second}");
    write(&first, "NASA appears first.");
    write(&second, "NASA appears second.");
    let mut config = config_for(RuleId::Acr002);
    config.rules.acr002.min_occurrences = 2;

    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn acr002_counts_across_included_files_and_reports_first_reading_order_occurrence() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let first = dir.path().join("first.tex");
    let second = dir.path().join("second.tex");
    write(&main, "\\input{first}\\input{second}");
    write(&first, "NASA appears first.");
    write(&second, "NASA appears second.");
    let mut config = config_for(RuleId::Acr002);
    config.rules.acr002.min_occurrences = 3;

    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].span.file, first.canonicalize().unwrap());
    let source = document.source(&diagnostics[0].span.file).unwrap();
    assert_eq!(
        &source.text[diagnostics[0].span.start..diagnostics[0].span.end],
        "NASA"
    );
}

#[test]
fn mixed_language_uses_effective_character_threshold() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "中文 AB。");
    let mut config = config_for(RuleId::Style001);
    config.rules.style001.max_chars = 2;
    config.rules.style001.max_english_words = 99;

    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("3 effective characters; max is 2")
    );
}

#[test]
fn acronym_and_term_matches_point_to_exact_logical_source_ranges() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "前缀\\textbf{NASA} 和 data set 后缀。");
    for (rule, expected) in [(RuleId::Acr001, "NASA"), (RuleId::Term001, "data set")] {
        let config = config_for(rule);
        let document = parser::parse(main.clone(), &config.latex).unwrap();
        let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
        assert_eq!(diagnostics.len(), 1, "{rule}");
        let source = document.source(&diagnostics[0].span.file).unwrap();
        assert_eq!(
            &source.text[diagnostics[0].span.start..diagnostics[0].span.end],
            expected
        );
    }
}
