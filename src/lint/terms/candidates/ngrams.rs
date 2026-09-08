use crate::{
    latex::span::LocatedRange,
    lint::{
        analysis::{DocumentAnalysis, SentenceUnit},
        terms::candidates::{
            model::{RawTermOccurrence, TermCandidateEvidence},
            text::{cjk_count, has_cjk},
        },
    },
    nlp::{PosTag, Token},
};
use std::{collections::BTreeMap, ops::Range};

const MIN_WINDOW: usize = 2;
const MAX_WINDOW: usize = 6;
const MIN_SCALARS: usize = 4;
const MAX_SCALARS: usize = 24;

pub(crate) fn collect(analysis: &DocumentAnalysis) -> Vec<RawTermOccurrence> {
    let mut grouped: BTreeMap<String, Vec<LocatedRange>> = BTreeMap::new();
    for paragraph in analysis.paragraphs() {
        for sentence in &paragraph.sentences {
            for window in candidate_windows(sentence) {
                grouped
                    .entry(window.surface)
                    .or_default()
                    .push(window.location);
            }
        }
    }
    repeated_occurrences(grouped)
}

fn candidate_windows(sentence: &SentenceUnit) -> impl Iterator<Item = NgramWindow> + '_ {
    candidate_ranges(sentence.tokens.len()).filter_map(|(start, end)| {
        sentence
            .tokens
            .get(start..end)
            .and_then(|tokens| ngram_window(sentence, tokens))
    })
}

fn candidate_ranges(token_count: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..token_count).flat_map(move |start| {
        let count = token_count
            .saturating_sub(start)
            .saturating_sub(MIN_WINDOW - 1)
            .min(MAX_WINDOW - MIN_WINDOW + 1);
        (0..count).map(move |offset| (start, start + MIN_WINDOW + offset))
    })
}

fn repeated_occurrences(grouped: BTreeMap<String, Vec<LocatedRange>>) -> Vec<RawTermOccurrence> {
    let mut occurrences = Vec::new();
    for (surface, locations) in grouped {
        if locations.len() < 2 {
            continue;
        }
        for location in &locations {
            occurrences.push(RawTermOccurrence {
                surface: surface.clone(),
                location: location.clone(),
                evidence: vec![TermCandidateEvidence::RepeatedNgram {
                    occurrences: locations.len(),
                }],
            });
        }
    }
    occurrences.sort_by(reading_order);
    occurrences
}

fn ngram_window(sentence: &SentenceUnit, tokens: &[Token]) -> Option<NgramWindow> {
    if !tokens.iter().all(is_nominal_token)
        || !is_sentence_local(sentence, tokens)
        || !is_contiguous(tokens)
    {
        return None;
    }
    if !tokens.iter().any(|token| cjk_count(&token.surface) >= 2) {
        return None;
    }
    let range = token_range(tokens)?;
    let relative = sentence_relative(sentence, &range)?;
    let surface = sentence.text.get(relative)?;
    let scalar_count = surface.chars().count();
    if !(MIN_SCALARS..=MAX_SCALARS).contains(&scalar_count) || !has_cjk(surface) {
        return None;
    }
    Some(NgramWindow {
        surface: surface.to_string(),
        location: LocatedRange {
            block: sentence.location.block,
            range,
        },
    })
}

fn is_nominal_token(token: &Token) -> bool {
    matches!(token.pos, PosTag::Noun | PosTag::Adjective)
}

fn is_sentence_local(sentence: &SentenceUnit, tokens: &[Token]) -> bool {
    tokens.iter().all(|token| {
        token.location.block == sentence.location.block
            && sentence.location.range.start <= token.location.range.start
            && token.location.range.start <= token.location.range.end
            && token.location.range.end <= sentence.location.range.end
    })
}

fn is_contiguous(tokens: &[Token]) -> bool {
    tokens
        .windows(2)
        .all(|pair| pair[0].location.range.end == pair[1].location.range.start)
}

fn token_range(tokens: &[Token]) -> Option<Range<usize>> {
    let first = tokens.first()?;
    let last = tokens.last()?;
    Some(first.location.range.start..last.location.range.end)
}

fn sentence_relative(sentence: &SentenceUnit, range: &Range<usize>) -> Option<Range<usize>> {
    Some(
        range.start.checked_sub(sentence.location.range.start)?
            ..range.end.checked_sub(sentence.location.range.start)?,
    )
}

fn reading_order(left: &RawTermOccurrence, right: &RawTermOccurrence) -> std::cmp::Ordering {
    left.location
        .position()
        .cmp(&right.location.position())
        .then(left.surface.cmp(&right.surface))
}

struct NgramWindow {
    surface: String,
    location: LocatedRange,
}

#[cfg(test)]
#[path = "ngrams_test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "ngrams_tests.rs"]
mod tests;
