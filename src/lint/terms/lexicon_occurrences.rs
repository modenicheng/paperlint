use crate::{
    latex::span::LocatedRange,
    text::lexicon::{FrozenMatchers, LexemeKind, LexemeSource, Lexicon},
};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

/// One lexicon-resolved term occurrence in the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconOccurrence {
    /// The surface actually appearing in the text.
    pub surface: String,
    /// The canonical form of the matched lexeme.
    pub canonical: String,
    pub kind: LexemeKind,
    pub source: LexemeSource,
    /// Whether the lexeme matched this surface with case sensitivity.
    pub case_sensitive: bool,
    /// Whether the lexeme marks terms that must be explained on first use.
    pub requires_explanation: bool,
    pub location: LocatedRange,
}

pub(super) struct LexiconScanner<'a> {
    lexicon: &'a Lexicon,
    matchers: Option<&'a FrozenMatchers>,
}

impl<'a> LexiconScanner<'a> {
    pub(super) const fn new(lexicon: &'a Lexicon, matchers: Option<&'a FrozenMatchers>) -> Self {
        Self { lexicon, matchers }
    }

    pub(super) fn collect(&self, text: &str, block_index: usize) -> Vec<LexiconOccurrence> {
        let Some(matchers) = self.matchers else {
            return Vec::new();
        };
        let lowered = text.to_ascii_lowercase();
        let mut collector = LexiconOccurrenceCollector::new(text, block_index, self.lexicon);
        for matched in matchers.exact.find_overlapping_iter(text) {
            collector.collect_match(
                matched.start()..matched.end(),
                &matchers.exact_key(&matched).canonical,
            );
        }
        for matched in matchers.folded.find_overlapping_iter(&lowered) {
            collector.collect_match(
                matched.start()..matched.end(),
                &matchers.folded_key(&matched).canonical,
            );
        }
        collector.into_occurrences()
    }
}

pub(super) fn first_occurrence_indexes(
    occurrences: &[LexiconOccurrence],
) -> HashMap<String, usize> {
    let mut indexes = HashMap::new();
    for (index, occurrence) in occurrences.iter().enumerate() {
        indexes.entry(occurrence.canonical.clone()).or_insert(index);
    }
    indexes
}

pub(super) fn sort_in_reading_order(occurrences: &mut [LexiconOccurrence]) {
    occurrences.sort_by(|left, right| {
        left.location
            .position()
            .cmp(&right.location.position())
            .then(left.location.range.end.cmp(&right.location.range.end))
            .then(left.canonical.cmp(&right.canonical))
    });
}

struct LexiconOccurrenceCollector<'a, 'text> {
    text: &'text str,
    block_index: usize,
    lexicon: &'a Lexicon,
    seen_spans: HashSet<(usize, usize, usize, &'a str)>,
    occurrences: Vec<LexiconOccurrence>,
}

impl<'a, 'text> LexiconOccurrenceCollector<'a, 'text> {
    fn new(text: &'text str, block_index: usize, lexicon: &'a Lexicon) -> Self {
        Self {
            text,
            block_index,
            lexicon,
            seen_spans: HashSet::new(),
            occurrences: Vec::new(),
        }
    }

    fn collect_match(&mut self, range: Range<usize>, expected_canonical: &str) {
        if !is_bounded_match(self.text, range.start, range.end) {
            return;
        }
        let surface = &self.text[range.clone()];
        let Some(lexeme) = self.lexicon.lookup(surface) else {
            return;
        };
        if lexeme.canonical != expected_canonical {
            return;
        }
        let span_key = (
            self.block_index,
            range.start,
            range.end,
            lexeme.canonical.as_str(),
        );
        if !self.seen_spans.insert(span_key) {
            return;
        }
        self.occurrences.push(LexiconOccurrence {
            surface: surface.to_string(),
            canonical: lexeme.canonical.clone(),
            kind: lexeme.kind,
            source: lexeme.source,
            case_sensitive: lexeme.case_sensitive,
            requires_explanation: lexeme.requires_explanation,
            location: LocatedRange {
                block: self.block_index,
                range,
            },
        });
    }

    fn into_occurrences(self) -> Vec<LexiconOccurrence> {
        self.occurrences
    }
}

fn is_bounded_match(text: &str, start: usize, end: usize) -> bool {
    let bytes = text.as_bytes();
    let identifier_byte = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    let before_ok = start == 0 || !identifier_byte(bytes[start - 1]);
    let after_ok = end >= bytes.len() || !identifier_byte(bytes[end]);
    before_ok && after_ok
}
