use super::{
    model::{RawTermOccurrence, TermCandidateEvidence},
    shapes::{collect_ascii_shapes, collect_mixed_script},
};
use crate::{
    latex::span::LocatedRange,
    lint::analysis::SentenceUnit,
    nlp::{PosTag, Token},
    text::Language,
};

fn sentence(text: &str, tokens: Vec<Token>) -> SentenceUnit {
    SentenceUnit {
        text: text.to_string(),
        language: Language::Mixed,
        location: LocatedRange {
            block: 2,
            range: 10..10 + text.len(),
        },
        tokens,
    }
}

fn token(surface: &str, pos: PosTag, start: usize) -> Token {
    Token {
        surface: surface.to_string(),
        pos,
        location: LocatedRange {
            block: 2,
            range: 10 + start..10 + start + surface.len(),
        },
    }
}

fn surfaces(occurrences: &[RawTermOccurrence]) -> Vec<&str> {
    occurrences
        .iter()
        .map(|occurrence| occurrence.surface.as_str())
        .collect()
}

#[test]
fn extracts_whole_ascii_shape_occurrences_with_exact_ranges() {
    let found = collect_ascii_shapes(&sentence(
        "The GraphRAG2 methodABCx ABC_method LLM works.",
        Vec::new(),
    ));

    assert_eq!(surfaces(&found), ["GraphRAG2", "methodABCx", "LLM"]);
    assert_eq!(found[0].location.range, 14..23);
    assert_eq!(
        found[0].evidence,
        [
            TermCandidateEvidence::CamelOrPascalCase,
            TermCandidateEvidence::LetterDigit
        ]
    );
}

#[test]
fn skips_sentence_initial_ordinary_pascal_words() {
    let found = collect_ascii_shapes(&sentence("This method uses GraphRAG.", Vec::new()));

    assert_eq!(surfaces(&found), ["GraphRAG"]);
}

#[test]
fn extracts_first_ascii_pascal_word_after_chinese_prose() {
    let text = "本文采用 Bert 方法。";
    let found = collect_ascii_shapes(&sentence(text, Vec::new()));

    assert_eq!(surfaces(&found), ["Bert"]);
    assert_eq!(found[0].location.range, 23..27);
    assert_eq!(
        found[0].evidence,
        [TermCandidateEvidence::CamelOrPascalCase]
    );
    assert_eq!(
        &text[found[0].location.range.start - 10..found[0].location.range.end - 10],
        "Bert"
    );
}

#[test]
fn does_not_extract_contained_shapes_from_separator_identifiers() {
    let found = collect_ascii_shapes(&sentence("ABC_method ABC-method", Vec::new()));

    assert!(found.is_empty());
}

#[test]
fn mixed_script_requires_one_shaped_ascii_component() {
    let text = "a中B Graph模型";
    let found = collect_mixed_script(&sentence(
        text,
        vec![
            token("a", PosTag::Other, 0),
            token("中", PosTag::Other, 1),
            token("B", PosTag::Other, 4),
            token("Graph", PosTag::Other, 6),
            token("模型", PosTag::Other, 11),
        ],
    ));

    assert_eq!(surfaces(&found), ["Graph模型"]);
    assert_eq!(found[0].location.range, 16..27);
}

#[test]
fn mixed_script_runs_stop_at_gaps_punctuation_and_six_tokens() {
    let text = "使用GraphRAG2模型，使用 FastText 方法提升效果指标结果。";
    let found = collect_mixed_script(&sentence(
        text,
        vec![
            token("使用", PosTag::Other, 0),
            token("GraphRAG2", PosTag::Other, 6),
            token("模型", PosTag::Other, 15),
            token("，", PosTag::Punctuation, 21),
            token("使用", PosTag::Other, 24),
            token("FastText", PosTag::Other, 31),
            token("方法", PosTag::Other, 40),
            token("提升", PosTag::Other, 46),
            token("效果", PosTag::Other, 52),
            token("指标", PosTag::Other, 58),
            token("结果", PosTag::Other, 64),
        ],
    ));

    assert_eq!(
        surfaces(&found),
        ["使用GraphRAG2", "使用GraphRAG2模型", "GraphRAG2模型"]
    );
}

#[test]
fn mixed_script_runs_are_six_token_bounded_and_deterministic() {
    let text = "GraphRAG2甲乙丙丁戊己";
    let unit = sentence(
        text,
        vec![
            token("GraphRAG2", PosTag::Other, 0),
            token("甲", PosTag::Other, 9),
            token("乙", PosTag::Other, 12),
            token("丙", PosTag::Other, 15),
            token("丁", PosTag::Other, 18),
            token("戊", PosTag::Other, 21),
            token("己", PosTag::Other, 24),
        ],
    );
    let first = collect_mixed_script(&unit);
    let second = collect_mixed_script(&unit);

    assert_eq!(first, second);
    assert!(surfaces(&first).contains(&"GraphRAG2甲乙丙丁戊"));
    assert!(!surfaces(&first).contains(&text));
    assert!(first.iter().all(|occurrence| &unit.text
        [occurrence.location.range.start - 10..occurrence.location.range.end - 10]
        == occurrence.surface.as_str()));
}
