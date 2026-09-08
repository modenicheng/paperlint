use paperlint::{
    config::{DefaultConfig, LatexConfig, Level, LexiconEntryConfig},
    latex::parser,
    lint::context::{LintContext, LocatedRange},
    nlp::{JiebaAnalyzer, LexicalAnalyzer, PosTag},
    rule_id::RuleId,
    text::lexicon::{LexemeKind, Lexicon},
};
use std::{fs, path::Path};
use tempfile::tempdir;

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn workspace_config() -> paperlint::config::PaperlintConfig {
    let mut config = DefaultConfig::load();
    config.rules.acr001.level = Level::Off;
    config.rules.acr002.level = Level::Off;
    config.rules.term001.level = Level::Off;
    config.rules.term002.level = Level::Off;
    config.rules.style001.level = Level::Off;
    config.rules.case001.level = Level::Off;
    config.rules.punc002.level = Level::Off;
    config
}

#[test]
fn located_range_uses_logical_utf8_byte_offsets_when_positioned() {
    let text = "甲A乙";
    let range = text.find('乙').unwrap()..text.len();
    let located = LocatedRange { block: 3, range };

    assert_eq!(located.position().block, 3);
    assert_eq!(located.position().byte, "甲A".len());
    assert_eq!(&text[located.range], "乙");
}

#[test]
fn jieba_tokens_keep_logical_locations_when_input_has_offset() {
    let analyzer = JiebaAnalyzer::new();
    let text = "前缀 使用LLM进行推理。";
    let sentence_start = text.find("使用").unwrap();
    let analysis = analyzer.analyze(
        &text[sentence_start..],
        LocatedRange {
            block: 2,
            range: sentence_start..text.len(),
        },
    );

    assert_eq!(analysis.location.block, 2);
    assert_eq!(analysis.location.range, sentence_start..text.len());
    for token in &analysis.tokens {
        assert_eq!(
            token.surface,
            text[token.location.range.clone()],
            "token should point into the original logical text"
        );
    }
    assert!(
        analysis
            .tokens
            .windows(2)
            .all(|tokens| tokens[0].location.range.end <= tokens[1].location.range.start)
    );
}

#[test]
fn jieba_injects_workspace_canonical_and_alias_surfaces_only() {
    let mut config = workspace_config();
    config.lexicon.entries = vec![
        LexiconEntryConfig {
            canonical: "跨模态语义蒸馏网络".to_string(),
            aliases: vec!["CMSDN".to_string()],
            kind: LexemeKind::Term,
            case_sensitive: None,
            requires_explanation: false,
        },
        LexiconEntryConfig {
            canonical: "含 空格".to_string(),
            aliases: vec!["也 有空格".to_string()],
            kind: LexemeKind::Term,
            case_sensitive: None,
            requires_explanation: false,
        },
    ];
    let lexicon = Lexicon::from_config(&config);
    let analyzer = JiebaAnalyzer::with_workspace_lexicon(&lexicon);

    let text = "我们使用跨模态语义蒸馏网络，也比较CMSDN。";
    let analysis = analyzer.analyze(
        text,
        LocatedRange {
            block: 0,
            range: 0..text.len(),
        },
    );

    let technical_terms: Vec<_> = analysis
        .tokens
        .iter()
        .filter(|token| token.pos == PosTag::Noun)
        .map(|token| token.surface.as_str())
        .collect();
    assert!(technical_terms.contains(&"跨模态语义蒸馏网络"));
    assert!(technical_terms.contains(&"CMSDN"));
    assert!(
        !analysis
            .tokens
            .iter()
            .any(|token| token.surface == "含 空格")
    );
}

#[test]
fn lint_context_builds_shared_analysis_with_source_mappable_tokens() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    write(&main, "\\input{child}\n\n主文件第二段。");
    write(&child, "前言\\textbf{跨模态语义蒸馏网络}用于测试。");
    let mut config = workspace_config();
    config.lexicon.entries = vec![LexiconEntryConfig {
        canonical: "跨模态语义蒸馏网络".to_string(),
        aliases: Vec::new(),
        kind: LexemeKind::Term,
        case_sensitive: None,
        requires_explanation: false,
    }];
    let document = parser::parse(main, &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);

    let paragraphs = context.analysis().paragraphs();
    assert_eq!(paragraphs.len(), document.blocks.len());
    assert_eq!(paragraphs[0].block, 0);
    let sentence = &paragraphs[0].sentences[0];
    assert_eq!(
        &document.blocks[0].text[sentence.location.range.clone()],
        sentence.text
    );
    let token = sentence
        .tokens
        .iter()
        .find(|token| token.surface == "跨模态语义蒸馏网络")
        .unwrap();
    let span = context.span_of(&token.location).unwrap();
    let source = document.source(&span.file).unwrap();
    assert_eq!(&source.text[span.start..span.end], "跨模态语义蒸馏网络");
    assert_eq!(span.file, child.canonicalize().unwrap());
}

#[test]
fn lint_context_skips_tokens_that_cross_unmappable_logical_gaps() {
    let block = paperlint::latex::span::TextBlock {
        text: "跨模态语义蒸馏网络".to_string(),
        mappings: vec![],
    };
    let document = paperlint::latex::parser::Document {
        entry: std::path::PathBuf::from("<memory>"),
        root: std::path::PathBuf::from("<memory>"),
        blocks: vec![block],
        sources: Vec::new(),
    };
    let config = workspace_config();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);

    let token = &context.analysis().paragraphs()[0].sentences[0].tokens[0];
    assert!(context.span_of(&token.location).is_none());
}

#[test]
fn style001_uses_shared_sentence_analysis_for_latex_gap_spans() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    let sentence = format!("甲\\textbf{{{}}}。", "乙".repeat(5));
    write(&main, "\\input{child}");
    write(&child, &sentence);
    let mut config = workspace_config();
    config.rules.style001.level = Level::Warning;
    config.rules.style001.max_chars = 5;

    let document = parser::parse(main, &config.latex).unwrap();
    let diagnostics = paperlint::lint::engine::RuleEngine::new(
        paperlint::lint::registry::RuleRegistry::default(),
    )
    .run(&document, &config);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule, RuleId::Style001);
    let source = document.source(&diagnostics[0].span.file).unwrap();
    assert_eq!(
        &source.text[diagnostics[0].span.start..diagnostics[0].span.end],
        sentence
    );
}

#[test]
fn empty_latex_config_is_enough_for_document_analysis() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "一句话。");
    let mut config = workspace_config();
    config.latex = LatexConfig {
        ignore_environments: Vec::new(),
    };
    let document = parser::parse(main, &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);

    assert_eq!(
        LintContext::new(&document, &config, &lexicon)
            .analysis()
            .paragraphs()
            .len(),
        1
    );
}
