use crate::{
    latex::span::LocatedRange,
    lint::{
        analysis::{DocumentAnalysis, SentenceUnit},
        terms::candidates::{
            model::{RawTermOccurrence, TermCandidateEvidence},
            text::{has_ascii_shape, has_cjk, has_punctuation},
        },
    },
    nlp::{PosTag, Token},
};
use std::ops::Range;

pub(crate) fn collect(analysis: &DocumentAnalysis) -> Vec<RawTermOccurrence> {
    let mut occurrences = Vec::new();
    for paragraph in analysis.paragraphs() {
        for sentence in &paragraph.sentences {
            occurrences.extend(collect_ascii_shapes(sentence));
            occurrences.extend(collect_mixed_script(sentence));
        }
    }
    occurrences.sort_by(reading_order);
    occurrences
}

pub(super) fn collect_ascii_shapes(sentence: &SentenceUnit) -> Vec<RawTermOccurrence> {
    let mut occurrences = Vec::new();
    for range in ascii_identifier_ranges(&sentence.text) {
        let surface = &sentence.text[range.clone()];
        let evidence = shape_evidence(
            surface,
            is_sentence_initial_ascii_word(&sentence.text, range.start),
        );
        if evidence.is_empty() {
            continue;
        }
        occurrences.push(raw(sentence, range, evidence));
    }
    occurrences
}

pub(super) fn collect_mixed_script(sentence: &SentenceUnit) -> Vec<RawTermOccurrence> {
    let mut occurrences = Vec::new();
    for start in 0..sentence.tokens.len() {
        for end in start..sentence.tokens.len().min(start + 6) {
            let tokens = &sentence.tokens[start..=end];
            if !is_valid_mixed_window(sentence, tokens) {
                break;
            }
            let range = token_range(tokens);
            let relative = sentence_relative(sentence, range);
            let surface = &sentence.text[relative.clone()];
            if has_cjk(surface) && ascii_component_has_shape(surface) {
                occurrences.push(raw(
                    sentence,
                    relative,
                    vec![TermCandidateEvidence::MixedScript],
                ));
            }
        }
    }
    occurrences
}

fn shape_evidence(surface: &str, is_sentence_initial: bool) -> Vec<TermCandidateEvidence> {
    if !surface.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Vec::new();
    }
    let mut evidence = Vec::new();
    if is_all_caps(surface) {
        evidence.push(TermCandidateEvidence::AllCaps);
    }
    if is_camel_or_pascal(surface, is_sentence_initial) {
        evidence.push(TermCandidateEvidence::CamelOrPascalCase);
    }
    if has_letter_and_digit(surface) {
        evidence.push(TermCandidateEvidence::LetterDigit);
    }
    evidence
}

fn ascii_identifier_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = None;
    for (index, ch) in text.char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            start.get_or_insert(index);
        } else if let Some(open) = start.take() {
            ranges.push(open..index);
        }
    }
    if let Some(open) = start {
        ranges.push(open..text.len());
    }
    ranges
}

fn is_sentence_initial_ascii_word(text: &str, start: usize) -> bool {
    !text[..start].chars().any(char::is_alphanumeric)
}

fn is_all_caps(surface: &str) -> bool {
    surface.chars().count() >= 2 && surface.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_camel_or_pascal(surface: &str, is_sentence_initial: bool) -> bool {
    if surface.starts_with(|ch: char| ch.is_ascii_digit()) {
        return false;
    }
    let mut previous_lower = false;
    for ch in surface.chars() {
        if previous_lower && ch.is_ascii_uppercase() {
            return true;
        }
        previous_lower = ch.is_ascii_lowercase();
    }
    !is_sentence_initial
        && surface
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        && surface.chars().any(|ch| ch.is_ascii_lowercase())
}

fn has_letter_and_digit(surface: &str) -> bool {
    surface.chars().any(|ch| ch.is_ascii_alphabetic())
        && surface.chars().any(|ch| ch.is_ascii_digit())
}

fn is_valid_mixed_window(sentence: &SentenceUnit, tokens: &[Token]) -> bool {
    if tokens
        .iter()
        .any(|token| matches!(token.pos, PosTag::Punctuation))
    {
        return false;
    }
    if tokens
        .windows(2)
        .any(|pair| pair[0].location.range.end != pair[1].location.range.start)
    {
        return false;
    }
    let relative = sentence_relative(sentence, token_range(tokens));
    !has_punctuation(&sentence.text[relative])
}

fn ascii_component_has_shape(surface: &str) -> bool {
    has_ascii_shape(surface)
}

fn token_range(tokens: &[Token]) -> Range<usize> {
    tokens[0].location.range.start..tokens[tokens.len() - 1].location.range.end
}

fn sentence_relative(sentence: &SentenceUnit, range: Range<usize>) -> Range<usize> {
    range.start - sentence.location.range.start..range.end - sentence.location.range.start
}

fn raw(
    sentence: &SentenceUnit,
    relative: Range<usize>,
    evidence: Vec<TermCandidateEvidence>,
) -> RawTermOccurrence {
    RawTermOccurrence {
        surface: sentence.text[relative.clone()].to_string(),
        location: LocatedRange {
            block: sentence.location.block,
            range: sentence.location.range.start + relative.start
                ..sentence.location.range.start + relative.end,
        },
        evidence,
    }
}

fn reading_order(left: &RawTermOccurrence, right: &RawTermOccurrence) -> std::cmp::Ordering {
    left.location
        .position()
        .cmp(&right.location.position())
        .then(left.surface.cmp(&right.surface))
}
