use super::text::is_context_boundary;
use crate::lint::analysis::SentenceUnit;
use std::ops::Range;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static DEFINITION_SCAN_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[derive(Default)]
pub(super) struct BoundaryCursor(usize);

#[derive(Default)]
pub(super) struct ForwardCursor {
    boundary: usize,
    predicate: usize,
}

pub(super) struct DefinitionIndex {
    text_len: usize,
    boundaries: Vec<usize>,
    boundary_ends: Vec<usize>,
    predicates: Vec<usize>,
    scalar_prefix: Vec<usize>,
    next_non_whitespace: Vec<usize>,
    previous_non_whitespace_end: Vec<usize>,
    token_starts_before: Vec<usize>,
    token_ends_through: Vec<usize>,
}

impl DefinitionIndex {
    pub(super) fn new(sentence: &SentenceUnit) -> Self {
        let text = &sentence.text;
        let text_len = text.len();
        let mut boundaries = Vec::new();
        let mut boundary_ends = Vec::new();
        let mut predicates = Vec::new();
        let mut scalar_prefix = vec![0; text_len + 1];
        let mut previous_non_whitespace_end = vec![0; text_len + 1];
        let mut scalar_count = 0;
        let mut previous_end = 0;

        for (position, ch) in text.char_indices() {
            record_scan(ch.len_utf8());
            scalar_prefix[position] = scalar_count;
            previous_non_whitespace_end[position] = previous_end;
            scalar_count += 1;
            if !ch.is_whitespace() {
                previous_end = position + ch.len_utf8();
            }
            if is_context_boundary(ch) {
                boundaries.push(position);
                boundary_ends.push(position + ch.len_utf8());
            }
            if is_definition_predicate(text, position) {
                predicates.push(position);
            }
        }
        scalar_prefix[text_len] = scalar_count;
        previous_non_whitespace_end[text_len] = previous_end;

        let mut next_non_whitespace = vec![text_len; text_len + 1];
        let mut next_start = text_len;
        for (position, ch) in text.char_indices().rev() {
            record_scan(ch.len_utf8());
            if !ch.is_whitespace() {
                next_start = position;
            }
            next_non_whitespace[position] = next_start;
        }
        let (token_starts_before, token_ends_through) = token_prefixes(sentence);

        Self {
            text_len,
            boundaries,
            boundary_ends,
            predicates,
            scalar_prefix,
            next_non_whitespace,
            previous_non_whitespace_end,
            token_starts_before,
            token_ends_through,
        }
    }

    pub(super) fn end_after(&self, start: usize, cursor: &mut ForwardCursor) -> usize {
        advance_to(&self.boundaries, start, &mut cursor.boundary);
        advance_to(&self.predicates, start, &mut cursor.predicate);
        let punctuation = self
            .boundaries
            .get(cursor.boundary)
            .copied()
            .unwrap_or(self.text_len);
        self.predicates
            .get(cursor.predicate)
            .copied()
            .filter(|position| *position < punctuation)
            .unwrap_or(punctuation)
    }

    pub(super) fn start_before(&self, end: usize, cursor: &mut BoundaryCursor) -> usize {
        while self
            .boundaries
            .get(cursor.0)
            .is_some_and(|position| *position < end)
        {
            cursor.0 += 1;
        }
        cursor
            .0
            .checked_sub(1)
            .and_then(|index| self.boundary_ends.get(index))
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn trimmed(&self, range: Range<usize>) -> Option<Range<usize>> {
        let start = *self.next_non_whitespace.get(range.start)?;
        let end = *self.previous_non_whitespace_end.get(range.end)?;
        (start < end && end <= range.end).then_some(start..end)
    }

    pub(super) fn scalar_count(&self, range: &Range<usize>) -> Option<usize> {
        let start = *self.scalar_prefix.get(range.start)?;
        let end = *self.scalar_prefix.get(range.end)?;
        end.checked_sub(start)
    }

    pub(super) fn token_count(&self, range: &Range<usize>) -> Option<usize> {
        let starts = *self.token_starts_before.get(range.end)?;
        let ends = *self.token_ends_through.get(range.start)?;
        starts.checked_sub(ends)
    }
}

fn advance_to(positions: &[usize], start: usize, cursor: &mut usize) {
    while positions
        .get(*cursor)
        .is_some_and(|position| *position < start)
    {
        *cursor += 1;
    }
}

fn is_definition_predicate(text: &str, position: usize) -> bool {
    [" 是", "是指", "指的是", "定义为"]
        .iter()
        .any(|predicate| text[position..].starts_with(predicate))
}

fn token_prefixes(sentence: &SentenceUnit) -> (Vec<usize>, Vec<usize>) {
    let text_len = sentence.text.len();
    let mut starts_before = vec![0_usize; text_len + 1];
    let mut ends_through = vec![0_usize; text_len + 1];
    for token in &sentence.tokens {
        if token.location.block != sentence.location.block {
            continue;
        }
        let Some(start) = token
            .location
            .range
            .start
            .checked_sub(sentence.location.range.start)
        else {
            continue;
        };
        let Some(end) = token
            .location
            .range
            .end
            .checked_sub(sentence.location.range.start)
        else {
            continue;
        };
        if let (Some(start_count), Some(end_count)) =
            (starts_before.get_mut(start), ends_through.get_mut(end))
        {
            *start_count += 1;
            *end_count += 1;
        }
    }

    let mut started = 0;
    let mut ended = 0;
    for position in 0..=text_len {
        let starts_at_position = starts_before[position];
        starts_before[position] = started;
        started += starts_at_position;
        ended += ends_through[position];
        ends_through[position] = ended;
    }
    (starts_before, ends_through)
}

#[cfg(test)]
fn record_scan(bytes: usize) {
    DEFINITION_SCAN_BYTES.with(|count| count.set(count.get() + bytes));
}

#[cfg(not(test))]
const fn record_scan(_bytes: usize) {}

#[cfg(test)]
pub(super) fn reset_definition_scan_bytes() {
    DEFINITION_SCAN_BYTES.with(|count| count.set(0));
}

#[cfg(test)]
pub(super) fn definition_scan_bytes() -> usize {
    DEFINITION_SCAN_BYTES.with(Cell::get)
}
