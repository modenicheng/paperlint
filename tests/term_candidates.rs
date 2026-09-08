use paperlint::{
    config::{DefaultConfig, Level, LexiconEntryConfig},
    latex::parser,
    lint::{
        context::LintContext,
        terms::{TermCandidateEvidence, TermCandidateKindHint},
    },
    text::lexicon::{LexemeKind, Lexicon},
};
use std::{fs, path::Path};
use tempfile::tempdir;

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn config() -> paperlint::config::PaperlintConfig {
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
fn registry_exposes_unknown_candidates_with_logical_locations_and_source_slices() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    write(&main, "\\input{child}\n\nAI 已知。");
    write(
        &child,
        "所谓 GraphRAG2 是核心。大语言模型（Large Language Model，LLM）已声明。",
    );
    let config = config();
    let document = parser::parse(main, &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);

    let candidates = context.registry().term_candidates();
    let surfaces: Vec<_> = candidates
        .iter()
        .map(|candidate| candidate.surface())
        .collect();

    assert!(surfaces.contains(&"GraphRAG2"));
    assert!(!surfaces.contains(&"AI"));
    assert!(!surfaces.contains(&"大语言模型"));
    assert!(!surfaces.contains(&"Large Language Model"));
    assert!(!surfaces.contains(&"LLM"));

    let graph = candidates
        .iter()
        .find(|candidate| candidate.surface() == "GraphRAG2")
        .unwrap();
    let span = context.span_of(graph.location()).unwrap();
    let source = document.source(&span.file).unwrap();
    assert_eq!(&source.text[span.start..span.end], "GraphRAG2");
}

#[test]
fn standalone_registry_and_lint_context_return_same_candidate_data() {
    let config = config();
    let document = parser::parse_stdin(
        "GraphRAG2用于实验。GraphRAG2继续出现。".to_string(),
        &config.latex,
    )
    .unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);
    let standalone = paperlint::lint::terms::DocumentTermRegistry::new(&document, &lexicon);

    assert_eq!(
        context.registry().term_candidates(),
        standalone.term_candidates()
    );
}

#[test]
fn workspace_lexicon_entries_suppress_unknown_candidates() {
    let mut config = config();
    config.lexicon.entries = vec![LexiconEntryConfig {
        canonical: "GraphRAG2".to_string(),
        aliases: vec!["graphrag2".to_string()],
        kind: LexemeKind::Term,
        case_sensitive: Some(false),
        requires_explanation: false,
    }];
    let document =
        parser::parse_stdin("GraphRAG2和graphrag2都已知。".to_string(), &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);

    let surfaces: Vec<_> = context
        .registry()
        .term_candidates()
        .iter()
        .map(|candidate| candidate.surface())
        .collect();
    assert!(!surfaces.contains(&"GraphRAG2"));
    assert!(!surfaces.contains(&"graphrag2"));
}

#[test]
fn aggregates_duplicate_evidence_at_the_earliest_exact_surface() {
    let config = config();
    let text = "所谓 GraphRAG2 是核心。GraphRAG2用于实验。";
    let document = parser::parse_stdin(text.to_string(), &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);
    let matches: Vec<_> = context
        .registry()
        .term_candidates()
        .iter()
        .filter(|candidate| candidate.surface() == "GraphRAG2")
        .collect();

    assert_eq!(matches.len(), 1);
    let graph = matches[0];
    assert_eq!(graph.location().range, 7..16);
    assert_eq!(
        graph.evidence(),
        [
            TermCandidateEvidence::CamelOrPascalCase,
            TermCandidateEvidence::LetterDigit,
            TermCandidateEvidence::DefinitionContext,
        ]
    );
    assert_eq!(graph.score().value(), 100);
    assert_eq!(graph.kind_hint(), TermCandidateKindHint::DefinitionLike);
    let span = context.span_of(graph.location()).unwrap();
    let source = document.source(&span.file).unwrap();
    assert_eq!(&source.text[span.start..span.end], "GraphRAG2");
}

#[test]
fn suppresses_builtin_workspace_ignore_folded_and_declared_surfaces() {
    let mut config = config();
    config.rules.acr001.ignore.push("ZZZ".to_string());
    config.lexicon.entries = vec![
        LexiconEntryConfig {
            canonical: "GraphRAG2".to_string(),
            aliases: vec!["GraphAlias".to_string()],
            kind: LexemeKind::Term,
            case_sensitive: Some(false),
            requires_explanation: false,
        },
        LexiconEntryConfig {
            canonical: "跨模态语义蒸馏网络".to_string(),
            aliases: Vec::new(),
            kind: LexemeKind::Term,
            case_sensitive: Some(false),
            requires_explanation: false,
        },
    ];
    let text = concat!(
        "AI ZZZ GraphRAG2 GRAPHALIAS。“跨模态语义蒸馏网络”。",
        "大语言模型（Large Language Model，LLM）。",
        "“大语言模型”“Large Language Model”“LLM”。"
    );
    let document = parser::parse_stdin(text.to_string(), &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);
    let surfaces: Vec<_> = context
        .registry()
        .term_candidates()
        .iter()
        .map(|candidate| candidate.surface())
        .collect();

    for suppressed in [
        "AI",
        "ZZZ",
        "GraphRAG2",
        "GRAPHALIAS",
        "跨模态语义蒸馏网络",
        "大语言模型",
        "Large Language Model",
        "LLM",
    ] {
        assert!(!surfaces.contains(&suppressed), "unexpected {suppressed}");
    }
}

#[test]
fn production_analysis_groups_a_repeated_nominal_phrase_once() {
    let config = config();
    let phrase = "神经网络模型";
    let text = format!("{phrase}提升性能。{phrase}用于比较。");
    let document = parser::parse_stdin(text, &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);
    let matches: Vec<_> = context
        .registry()
        .term_candidates()
        .iter()
        .filter(|candidate| candidate.surface() == phrase)
        .collect();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location().range, 0..phrase.len());
    assert_eq!(
        matches[0].evidence(),
        [TermCandidateEvidence::RepeatedNgram { occurrences: 2 }]
    );
}

#[test]
fn same_surface_across_include_and_formatting_gap_keeps_first_source_span() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    write(
        &main,
        "\\input{child}\n\n本文采用 Bert 方法。GraphRAG2在主文使用。",
    );
    write(&child, "Graph\\textbf{RAG}2在子文件首先使用。");
    let config = config();
    let document = parser::parse(main, &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let context = LintContext::new(&document, &config, &lexicon);
    let matches: Vec<_> = context
        .registry()
        .term_candidates()
        .iter()
        .filter(|candidate| candidate.surface() == "GraphRAG2")
        .collect();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location().block, 0);
    let graph_index = context
        .registry()
        .term_candidates()
        .iter()
        .position(|candidate| candidate.surface() == "GraphRAG2")
        .unwrap();
    let bert_index = context
        .registry()
        .term_candidates()
        .iter()
        .position(|candidate| candidate.surface() == "Bert")
        .unwrap();
    assert!(graph_index < bert_index);
    assert_eq!(
        context.registry().term_candidates()[bert_index]
            .location()
            .block,
        1
    );
    let span = context.span_of(matches[0].location()).unwrap();
    assert_eq!(span.file, child.canonicalize().unwrap());
    let source = document.source(&span.file).unwrap();
    assert_eq!(&source.text[span.start..span.end], "Graph\\textbf{RAG}2");
}
