use super::*;
use crate::{lint::analysis::ParagraphUnit, text::Language};

pub(super) fn analysis(paragraphs: Vec<ParagraphUnit>) -> DocumentAnalysis {
    DocumentAnalysis::from_paragraphs_for_tests(paragraphs)
}

pub(super) fn paragraph(block: usize, sentences: Vec<SentenceUnit>) -> ParagraphUnit {
    ParagraphUnit {
        block,
        location: located(block, 0..0),
        sentences,
    }
}

pub(super) fn sentence_at(location: LocatedRange, text: &str, tokens: Vec<Token>) -> SentenceUnit {
    SentenceUnit {
        text: text.to_string(),
        language: Language::Chinese,
        location,
        tokens,
    }
}

pub(super) fn one_token_sentence(block: usize, start: usize, surface: &str) -> SentenceUnit {
    let end = start + surface.len();
    sentence_at(
        located(block, start..end),
        surface,
        vec![token(surface, PosTag::Noun, located(block, start..end))],
    )
}

pub(super) fn token(surface: &str, pos: PosTag, location: LocatedRange) -> Token {
    Token {
        surface: surface.to_string(),
        pos,
        location,
    }
}

pub(super) fn located(block: usize, range: Range<usize>) -> LocatedRange {
    LocatedRange { block, range }
}

pub(super) fn phrase_tokens(block: usize, offset: usize) -> Vec<Token> {
    vec![
        token("跨模态", PosTag::Noun, located(block, offset..offset + 9)),
        token(
            "语义",
            PosTag::Noun,
            located(block, offset + 9..offset + 15),
        ),
        token(
            "蒸馏",
            PosTag::Noun,
            located(block, offset + 15..offset + 21),
        ),
        token(
            "网络",
            PosTag::Noun,
            located(block, offset + 21..offset + 27),
        ),
    ]
}

pub(super) fn repeated_pair(first: &str, second: &str, second_pos: PosTag) -> SentenceUnit {
    repeated_tokens_with_pos(&[first, second], second_pos)
}

pub(super) fn repeated_tokens(surfaces: &[&str], pos: PosTag) -> SentenceUnit {
    repeated_tokens_with_pos(surfaces, pos)
}

fn repeated_tokens_with_pos(surfaces: &[&str], last_pos: PosTag) -> SentenceUnit {
    let one_pass: String = surfaces.concat();
    let text = one_pass.repeat(2);
    let mut tokens = Vec::new();
    let mut start = 0;
    for _ in 0..2 {
        for (index, surface) in surfaces.iter().enumerate() {
            let end = start + surface.len();
            let pos = if index + 1 == surfaces.len() {
                last_pos.clone()
            } else {
                PosTag::Noun
            };
            tokens.push(token(surface, pos, located(0, start..end)));
            start = end;
        }
    }
    sentence_at(located(0, 0..text.len()), &text, tokens)
}

pub(super) fn pair_sentence(first: &str, second: &str, start: usize) -> SentenceUnit {
    let text = format!("{first}{second}");
    let split = start + first.len();
    sentence_at(
        located(0, start..start + text.len()),
        &text,
        vec![
            token(first, PosTag::Noun, located(0, start..split)),
            token(second, PosTag::Noun, located(0, split..start + text.len())),
        ],
    )
}

pub(super) fn collect_one(sentence: SentenceUnit) -> Vec<RawTermOccurrence> {
    collect(&analysis(vec![paragraph(0, vec![sentence])]))
}

pub(super) fn matching<'a>(
    occurrences: &'a [RawTermOccurrence],
    surface: &str,
) -> Vec<&'a RawTermOccurrence> {
    occurrences
        .iter()
        .filter(|occurrence| occurrence.surface == surface)
        .collect()
}

pub(super) fn assert_repetition(occurrences: &[&RawTermOccurrence], count: usize) {
    assert!(occurrences.iter().all(|occurrence| {
        occurrence.evidence == [TermCandidateEvidence::RepeatedNgram { occurrences: count }]
    }));
}
