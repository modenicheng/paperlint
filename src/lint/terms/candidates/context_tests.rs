use super::{
    context::collect_sentence,
    context_index::{definition_scan_bytes, reset_definition_scan_bytes},
    model::{RawTermOccurrence, TermCandidateEvidence},
};
use crate::{
    latex::span::LocatedRange,
    lint::analysis::{DocumentAnalysis, ParagraphUnit, SentenceUnit},
    nlp::{PosTag, Token},
    text::Language,
};

fn sentence(text: &str) -> SentenceUnit {
    SentenceUnit {
        text: text.to_string(),
        language: Language::Mixed,
        location: LocatedRange {
            block: 1,
            range: 20..20 + text.len(),
        },
        tokens: Vec::<Token>::new(),
    }
}

fn sentence_with_tokens(text: &str, surfaces: &[&str]) -> SentenceUnit {
    let mut unit = sentence(text);
    let mut search_start = 0;
    for surface in surfaces {
        let Some(relative) = text[search_start..].find(surface) else {
            panic!("test token must occur in order");
        };
        let start = search_start + relative;
        unit.tokens.push(Token {
            surface: (*surface).to_string(),
            pos: PosTag::Other,
            location: LocatedRange {
                block: 1,
                range: 20 + start..20 + start + surface.len(),
            },
        });
        search_start = start + surface.len();
    }
    unit
}

fn surfaces(occurrences: &[RawTermOccurrence]) -> Vec<&str> {
    occurrences
        .iter()
        .map(|occurrence| occurrence.surface.as_str())
        .collect()
}

#[test]
fn extracts_delimited_emphasis_without_delimiters() {
    let text = "本文采用“跨模态语义蒸馏网络”和(GraphRAG2)。";
    let occurrences = collect_sentence(&sentence(text));

    assert_eq!(surfaces(&occurrences), ["跨模态语义蒸馏网络", "GraphRAG2"]);
    assert_eq!(
        occurrences[0].location,
        LocatedRange {
            block: 1,
            range: 35..62,
        }
    );
    assert_eq!(occurrences[0].evidence, [TermCandidateEvidence::Emphasis]);
    assert_eq!(occurrences[1].evidence, [TermCandidateEvidence::Emphasis]);
}

#[test]
fn extracts_all_supported_delimiter_pairs_and_ignores_apostrophes() {
    let text = "“甲” ‘乙’ \"丙\" '丁' （戊） (己) isn't 'GraphRAG2'";
    let occurrences = collect_sentence(&sentence(text));

    assert_eq!(
        surfaces(&occurrences),
        ["甲", "乙", "丙", "丁", "戊", "己", "GraphRAG2"]
    );
    assert!(occurrences.iter().all(|occurrence| {
        &text[occurrence.location.range.start - 20..occurrence.location.range.end - 20]
            == occurrence.surface.as_str()
    }));
}

#[test]
fn extracts_definition_forms_without_triggers() {
    let text = "所谓 跨模态语义蒸馏网络 是核心。GraphRAG2 refers to graph retrieval.";
    let occurrences = collect_sentence(&sentence(text));

    assert_eq!(surfaces(&occurrences), ["跨模态语义蒸馏网络", "GraphRAG2"]);
    assert!(
        occurrences
            .iter()
            .all(|occurrence| occurrence.evidence == [TermCandidateEvidence::DefinitionContext])
    );
    assert_eq!(occurrences[0].location.range, 27..54);
}

#[test]
fn extracts_every_definition_trigger_with_exact_content() {
    let text = concat!(
        "甲是指一、乙指的是二、丙定义为三。",
        "GraphRAG2 refers to one, FastText is defined as two, ",
        "ABC denotes three, LLM stands for four."
    );
    let occurrences = collect_sentence(&sentence(text));

    assert_eq!(
        surfaces(&occurrences),
        ["甲", "乙", "丙", "GraphRAG2", "FastText", "ABC", "LLM"]
    );
    assert!(occurrences.iter().all(|occurrence| {
        &text[occurrence.location.range.start - 20..occurrence.location.range.end - 20]
            == occurrence.surface.as_str()
    }));
    assert!(
        occurrences
            .iter()
            .all(|occurrence| occurrence.evidence == [TermCandidateEvidence::DefinitionContext])
    );
}

#[test]
fn rejects_unterminated_nested_punctuation_only_and_long_captures() {
    let long = "一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九";
    let text = format!("“外层“内层”。所谓 ，。X 是 Y。{long} 是指 过长。");
    let occurrences = collect_sentence(&sentence(&text));

    assert!(surfaces(&occurrences).is_empty());
}

#[test]
fn context_content_has_an_inclusive_48_scalar_limit() {
    let accepted = "甲".repeat(48);
    let rejected = "乙".repeat(49);
    let text = format!("“{accepted}”和“{rejected}”");
    let occurrences = collect_sentence(&sentence(&text));

    assert_eq!(surfaces(&occurrences), [accepted.as_str()]);
}

#[test]
fn rejects_empty_punctuation_only_and_emoji_only_content() {
    let occurrences = collect_sentence(&sentence("“” “，。” “🚀” “GraphRAG2🚀”"));

    assert_eq!(surfaces(&occurrences), ["GraphRAG2🚀"]);
}

#[test]
fn generic_colon_and_plain_chinese_is_do_not_define_terms() {
    let occurrences = collect_sentence(&sentence("方法：简单。模型 是 有效的。"));

    assert!(occurrences.is_empty());
}

#[test]
fn rejects_unbounded_context_token_runs() {
    let text = "“Alpha Beta Gamma Delta Epsilon Zeta Eta”";
    let unit = sentence_with_tokens(
        text,
        &["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta"],
    );

    assert!(collect_sentence(&unit).is_empty());
}

#[test]
fn definition_triggers_must_be_explicit_and_bounded() {
    let occurrences = collect_sentence(&sentence("无所谓 X。predenotes value."));

    assert!(occurrences.is_empty());
}

#[test]
fn context_collection_is_sentence_local_and_deterministic() {
    let first = sentence("本文称“未闭合。");
    let second = sentence("下一句”不是候选。");
    let analysis = DocumentAnalysis::from_paragraphs_for_tests(vec![ParagraphUnit {
        block: 1,
        location: LocatedRange {
            block: 1,
            range: 20..20 + first.text.len() + second.text.len(),
        },
        sentences: vec![first, second],
    }]);

    let first_run = super::context::collect(&analysis);
    let second_run = super::context::collect(&analysis);
    assert!(first_run.is_empty());
    assert_eq!(first_run, second_run);
}

#[test]
fn definition_context_scan_is_linear_when_explicit_markers_are_dense() {
    // Given: every marker is explicit and no punctuation or predicate bounds its suffix.
    let text = format!("{}X", "所谓 ".repeat(512));
    reset_definition_scan_bytes();

    // When: definition contexts are collected through the production entry point.
    let occurrences = collect_sentence(&sentence(&text));
    let scanned_bytes = definition_scan_bytes();

    // Then: scan work stays within a constant multiple of the sentence length.
    assert_eq!(surfaces(&occurrences).last(), Some(&"X"));
    assert!(
        scanned_bytes <= text.len() * 8,
        "definition scan revisited suffixes: scanned {scanned_bytes} bytes for {} input bytes",
        text.len()
    );
}

#[test]
fn definition_context_keeps_boundary_and_earliest_predicate_semantics() {
    // Given: one marker is punctuation-bounded and one has competing predicates.
    let text = "所谓 Alpha，Beta。所谓 Gamma 是 Delta 是指 Epsilon。";

    // When: definition contexts are extracted.
    let occurrences = collect_sentence(&sentence(text));

    // Then: punctuation and the earliest predicate end their respective captures.
    assert_eq!(
        surfaces(&occurrences),
        ["Alpha", "所谓 Gamma 是 Delta", "Gamma"]
    );
    assert!(occurrences.iter().all(|occurrence| {
        &text[occurrence.location.range.start - 20..occurrence.location.range.end - 20]
            == occurrence.surface.as_str()
    }));
}

#[test]
fn definition_context_keeps_unicode_scalar_limit_without_a_boundary() {
    // Given: adjacent definition captures contain exactly 48 and 49 Unicode scalars.
    let accepted = "甲".repeat(48);
    let rejected = "乙".repeat(49);
    let text = format!("所谓 {accepted} 是核心。所谓 {rejected} 是核心");

    // When: definition contexts are extracted.
    let occurrences = collect_sentence(&sentence(&text));

    // Then: the inclusive scalar limit accepts only the 48-scalar surface.
    assert_eq!(surfaces(&occurrences), [accepted.as_str()]);
}
