use super::{
    context_index::{BoundaryCursor, DefinitionIndex, ForwardCursor},
    model::{RawTermOccurrence, TermCandidateEvidence},
    text::{
        is_context_boundary, is_context_punctuation_or_space, is_context_term_char, trim_range,
        within_token_limit,
    },
};
use crate::{
    latex::span::LocatedRange,
    lint::analysis::{DocumentAnalysis, SentenceUnit},
};

pub(crate) fn collect(analysis: &DocumentAnalysis) -> Vec<RawTermOccurrence> {
    let mut occurrences = Vec::new();
    for paragraph in analysis.paragraphs() {
        for sentence in &paragraph.sentences {
            occurrences.extend(collect_sentence(sentence));
        }
    }
    occurrences.sort_by_key(|occurrence| occurrence.location.position());
    occurrences
}

pub(super) fn collect_sentence(sentence: &SentenceUnit) -> Vec<RawTermOccurrence> {
    let mut collector = ContextCollector::new(sentence);
    collector.collect_delimited();
    collector.collect_definition_contexts();
    collector.finish()
}

struct ContextCollector<'a> {
    sentence: &'a SentenceUnit,
    occurrences: Vec<RawTermOccurrence>,
}

impl<'a> ContextCollector<'a> {
    fn new(sentence: &'a SentenceUnit) -> Self {
        Self {
            sentence,
            occurrences: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<RawTermOccurrence> {
        self.occurrences
            .sort_by_key(|occurrence| occurrence.location.position());
        self.occurrences
    }

    fn collect_delimited(&mut self) {
        for pair in delimiter_pairs() {
            let mut search_start = 0_usize;
            while let Some(open_start) =
                find_delimiter(&self.sentence.text, pair.open, search_start)
            {
                let content_start = open_start + pair.open.len();
                let Some(close_start) =
                    find_delimiter(&self.sentence.text, pair.close, content_start)
                else {
                    break;
                };
                self.push_trimmed(content_start..close_start, TermCandidateEvidence::Emphasis);
                search_start = close_start + pair.close.len();
            }
        }
    }

    fn collect_definition_contexts(&mut self) {
        let index = DefinitionIndex::new(self.sentence);
        self.collect_after_definition_marker(&index);
        for trigger in ["是指", "指的是", "定义为"] {
            self.collect_before_trigger(trigger, &index);
        }
        for trigger in ["refers to", "is defined as", "denotes", "stands for"] {
            self.collect_before_trigger(trigger, &index);
        }
    }

    fn collect_after_definition_marker(&mut self, index: &DefinitionIndex) {
        let trigger = "所谓";
        let mut search_start = 0_usize;
        let mut cursor = ForwardCursor::default();
        while let Some(relative) = self.sentence.text[search_start..].find(trigger) {
            let trigger_start = search_start + relative;
            let content_start = trigger_start + trigger.len();
            if !is_explicit_after_marker(&self.sentence.text, trigger_start) {
                search_start = content_start;
                continue;
            }
            let content_end = index.end_after(content_start, &mut cursor);
            self.push_definition(content_start..content_end, index);
            search_start = content_start;
        }
    }

    fn collect_before_trigger(&mut self, trigger: &str, index: &DefinitionIndex) {
        let mut search_start = 0_usize;
        let mut cursor = BoundaryCursor::default();
        while let Some(relative) = self.sentence.text[search_start..].find(trigger) {
            let trigger_start = search_start + relative;
            if !is_explicit_before_marker(&self.sentence.text, trigger_start, trigger) {
                search_start = trigger_start + trigger.len();
                continue;
            }
            let content_start = index.start_before(trigger_start, &mut cursor);
            self.push_definition(content_start..trigger_start, index);
            search_start = trigger_start + trigger.len();
        }
    }

    fn push_definition(&mut self, range: std::ops::Range<usize>, index: &DefinitionIndex) {
        let Some(trimmed) = index.trimmed(range) else {
            return;
        };
        let Some(surface) = self.sentence.text.get(trimmed.clone()) else {
            return;
        };
        if index.scalar_count(&trimmed).is_none_or(|count| count > 48)
            || index.token_count(&trimmed).is_none_or(|count| count > 6)
            || !valid_context_surface(surface)
        {
            return;
        }
        self.occurrences.push(RawTermOccurrence {
            surface: surface.to_string(),
            location: LocatedRange {
                block: self.sentence.location.block,
                range: self.sentence.location.range.start + trimmed.start
                    ..self.sentence.location.range.start + trimmed.end,
            },
            evidence: vec![TermCandidateEvidence::DefinitionContext],
        });
    }

    fn push_trimmed(&mut self, range: std::ops::Range<usize>, evidence: TermCandidateEvidence) {
        let Some(trimmed) = trim_range(&self.sentence.text, range) else {
            return;
        };
        let Some(surface) = self.sentence.text.get(trimmed.clone()) else {
            return;
        };
        if !valid_context_surface(surface) || !within_token_limit(self.sentence, &trimmed) {
            return;
        }
        self.occurrences.push(RawTermOccurrence {
            surface: surface.to_string(),
            location: LocatedRange {
                block: self.sentence.location.block,
                range: self.sentence.location.range.start + trimmed.start
                    ..self.sentence.location.range.start + trimmed.end,
            },
            evidence: vec![evidence],
        });
    }
}

fn find_delimiter(text: &str, delimiter: &str, start: usize) -> Option<usize> {
    let mut search_start = start;
    while let Some(relative) = text[search_start..].find(delimiter) {
        let position = search_start + relative;
        if delimiter != "'" || !is_apostrophe(text, position) {
            return Some(position);
        }
        search_start = position + delimiter.len();
    }
    None
}

fn is_apostrophe(text: &str, position: usize) -> bool {
    text[..position]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
        && text[position + 1..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
}

#[derive(Debug, Clone, Copy)]
struct DelimiterPair {
    open: &'static str,
    close: &'static str,
}

const fn delimiter_pairs() -> [DelimiterPair; 6] {
    [
        DelimiterPair {
            open: "“",
            close: "”",
        },
        DelimiterPair {
            open: "‘",
            close: "’",
        },
        DelimiterPair {
            open: "\"",
            close: "\"",
        },
        DelimiterPair {
            open: "'",
            close: "'",
        },
        DelimiterPair {
            open: "（",
            close: "）",
        },
        DelimiterPair {
            open: "(",
            close: ")",
        },
    ]
}

fn valid_context_surface(surface: &str) -> bool {
    surface.chars().count() <= 48
        && !surface.chars().all(is_context_punctuation_or_space)
        && !contains_delimiter(surface)
        && surface.chars().any(is_context_term_char)
}

fn contains_delimiter(surface: &str) -> bool {
    delimiter_pairs()
        .iter()
        .any(|candidate| surface.contains(candidate.open) || surface.contains(candidate.close))
}

fn is_explicit_after_marker(text: &str, start: usize) -> bool {
    text[..start].chars().next_back().is_none_or(|ch| {
        ch.is_whitespace() || is_context_boundary(ch) || matches!(ch, '(' | '（' | '"')
    })
}

fn is_explicit_before_marker(text: &str, start: usize, trigger: &str) -> bool {
    if trigger.is_ascii() {
        return text[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
            && text[start + trigger.len()..]
                .chars()
                .next()
                .is_none_or(|ch| ch.is_whitespace() || is_context_boundary(ch));
    }
    true
}
