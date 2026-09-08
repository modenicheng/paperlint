pub(super) fn has_ascii_shape(surface: &str) -> bool {
    ascii_identifier_ranges(surface)
        .into_iter()
        .any(|range| !shape_evidence(&surface[range]).is_empty())
}

pub(super) fn has_cjk(surface: &str) -> bool {
    surface.chars().any(is_cjk)
}

pub(super) fn cjk_count(surface: &str) -> usize {
    surface.chars().filter(|ch| is_cjk(*ch)).count()
}

pub(super) fn has_punctuation(surface: &str) -> bool {
    surface
        .chars()
        .any(|ch| ch.is_ascii_punctuation() || is_cjk_punctuation(ch))
}

fn shape_evidence(surface: &str) -> Vec<()> {
    if !surface.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Vec::new();
    }
    let mut evidence = Vec::new();
    if is_all_caps(surface) {
        evidence.push(());
    }
    if is_camel_or_pascal(surface) {
        evidence.push(());
    }
    if has_letter_and_digit(surface) {
        evidence.push(());
    }
    evidence
}

fn ascii_identifier_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
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

fn is_all_caps(surface: &str) -> bool {
    surface.chars().count() >= 2 && surface.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_camel_or_pascal(surface: &str) -> bool {
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
    surface
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
        && surface.chars().any(|ch| ch.is_ascii_lowercase())
}

fn has_letter_and_digit(surface: &str) -> bool {
    surface.chars().any(|ch| ch.is_ascii_alphabetic())
        && surface.chars().any(|ch| ch.is_ascii_digit())
}

fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{20000}'..='\u{2A6DF}' |
        '\u{2A700}'..='\u{2B73F}' |
        '\u{2B740}'..='\u{2B81F}' |
        '\u{2B820}'..='\u{2CEAF}' |
        '\u{F900}'..='\u{FAFF}' |
        '\u{2F800}'..='\u{2FA1F}'
    )
}

pub(super) fn trim_range(text: &str, range: Range<usize>) -> Option<Range<usize>> {
    let content = text.get(range.clone())?;
    let leading = content.len() - content.trim_start().len();
    let trailing = content.len() - content.trim_end().len();
    let start = range.start + leading;
    let end = range.end.saturating_sub(trailing);
    if start < end { Some(start..end) } else { None }
}

pub(super) fn within_token_limit(sentence: &SentenceUnit, range: &Range<usize>) -> bool {
    let absolute =
        sentence.location.range.start + range.start..sentence.location.range.start + range.end;
    sentence
        .tokens
        .iter()
        .filter(|token| {
            token.location.range.start < absolute.end && token.location.range.end > absolute.start
        })
        .take(7)
        .count()
        <= 6
}

pub(super) fn is_context_boundary(ch: char) -> bool {
    matches!(
        ch,
        '。' | '，' | ',' | ';' | '；' | '.' | '!' | '！' | '?' | '？' | ':' | '：' | '、'
    )
}

pub(super) fn is_context_term_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || is_cjk(ch)
}

pub(super) fn is_context_punctuation_or_space(ch: char) -> bool {
    ch.is_whitespace() || is_context_boundary(ch) || matches!(ch, '、' | '(' | ')' | '（' | '）')
}

fn is_cjk_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '。' | '，' | '！' | '？' | '；' | '：' | '、' | '（' | '）' | '“' | '”'
    )
}
use crate::lint::analysis::SentenceUnit;
use std::ops::Range;
