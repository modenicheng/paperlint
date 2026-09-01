//! PUNC002 — missing space between CJK and Latin/number.
//!
//! The rule scans each [`TextBlock`] for logically adjacent character pairs
//! where a CJK ideograph directly touches an ASCII Latin letter or digit.
//! Punctuation never forms a trigger pair, but it also does not mask the
//! other side of a token (`使用LLM。` still reports the `用|L` boundary).
//!
//! Boundaries are only reported when both characters are kept inside one
//! [`SourceMapping`]. When the parser drops source bytes between the two
//! characters (formatting commands such as `\allowbreak{}` or empty groups),
//! the logical adjacency is a parsing artifact and the pair is skipped.
//! Cross-mapping pairs whose source bytes are contiguous remain reportable.
//!
//! Self-contained by design: everything except the `Rule` impl lives here so
//! the rule can be re-wired onto a different trait without semantic drift.

use crate::{
    latex::span::TextBlock,
    lint::{context::LintContext, diagnostic::Diagnostic},
    rule_id::RuleId,
};
use regex::Regex;
use std::ops::Range;

pub struct Punc002;

const MESSAGE: &str = "missing space between CJK and Latin/number";

impl crate::rules::Rule for Punc002 {
    fn id(&self) -> RuleId {
        RuleId::Punc002
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.punc002;
        let document = context.document();
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let patterns = compile_ignore_patterns(&rule.ignore_patterns);
        let mut diagnostics = Vec::new();
        for (block_index, block) in document.blocks.iter().enumerate() {
            for range in block_findings(block, &patterns) {
                let Some(span) = context.span(block_index, range) else {
                    continue;
                };
                diagnostics.push(Diagnostic {
                    rule: self.id(),
                    severity: rule.level,
                    message: MESSAGE.to_string(),
                    span,
                });
            }
        }
        diagnostics.sort_by(|left, right| {
            left.span
                .file
                .cmp(&right.span.file)
                .then(left.span.start.cmp(&right.span.start))
                .then(left.span.end.cmp(&right.span.end))
        });
        diagnostics
    }
}

/// Compile configured ignore patterns, silently dropping invalid regexes so a
/// bad entry can never panic or abort a lint run.
fn compile_ignore_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

/// Ordered boundary findings for one block, in logical byte ranges.
///
/// A trigger pair is exempt when its two-character span overlaps any match of
/// an ignore pattern in the block's logical text. Matching against the whole
/// block (instead of only the span) lets phrases such as `第\d+章` exempt both
/// of their internal CJK/digit boundaries.
fn block_findings(block: &TextBlock, patterns: &[Regex]) -> Vec<Range<usize>> {
    let text = &block.text;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut findings = Vec::new();
    if chars.len() < 2 {
        return findings;
    }
    let exemptions: Vec<Range<usize>> = patterns
        .iter()
        .flat_map(|pattern| pattern.find_iter(text).map(|matched| matched.range()))
        .collect();
    for pair in chars.windows(2) {
        let (start, left) = pair[0];
        let (right_start, right) = pair[1];
        if !is_missing_space_boundary(left, right) {
            continue;
        }
        let end = right_start + right.len_utf8();
        let left_span = start..right_start;
        let right_span = right_start..end;
        if !boundary_is_supported(&block.mappings, &left_span, &right_span) {
            continue;
        }
        if exemptions
            .iter()
            .any(|range| start < range.end && range.start < end)
        {
            continue;
        }
        findings.push(start..end);
    }
    findings
}

/// Frozen mapping semantics: a boundary is reportable when both characters
/// share one source mapping, or when they sit in different mappings whose
/// source bytes are contiguous (same file, `left.end == right.start`). A gap
/// means the parser dropped source bytes between the characters (formatting
/// commands, empty groups), so the adjacency is a parsing artifact. Pairs
/// whose exact source span cannot be mapped are dropped later by
/// `Document::source_span` returning `None`.
fn boundary_is_supported(
    mappings: &[crate::latex::span::SourceMapping],
    left: &Range<usize>,
    right: &Range<usize>,
) -> bool {
    let Some(left_mapping) = covering_mapping(mappings, left) else {
        return false;
    };
    let Some(right_mapping) = covering_mapping(mappings, right) else {
        return false;
    };
    if std::ptr::eq(left_mapping, right_mapping) {
        return true;
    }
    left_mapping.source.file == right_mapping.source.file
        && left_mapping.source.end == right_mapping.source.start
}

fn covering_mapping<'a>(
    mappings: &'a [crate::latex::span::SourceMapping],
    range: &Range<usize>,
) -> Option<&'a crate::latex::span::SourceMapping> {
    mappings
        .iter()
        .find(|mapping| mapping.logical.start <= range.start && range.end <= mapping.logical.end)
}

fn is_missing_space_boundary(left: char, right: char) -> bool {
    (is_cjk(left) && is_ascii_word(right)) || (is_ascii_word(left) && is_cjk(right))
}

fn is_ascii_word(character: char) -> bool {
    character.is_ascii_alphabetic() || character.is_ascii_digit()
}

fn is_cjk(character: char) -> bool {
    matches!(character,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::latex::span::{SourceMapping, Span};
    use std::path::PathBuf;

    fn mapping(logical: Range<usize>, source: Range<usize>) -> SourceMapping {
        SourceMapping {
            logical,
            source: Span {
                file: PathBuf::new(),
                start: source.start,
                end: source.end,
                line: 0,
                column: 0,
            },
        }
    }

    fn slices<'a>(block: &'a TextBlock, findings: &'a [Range<usize>]) -> Vec<&'a str> {
        findings
            .iter()
            .map(|range| &block.text[range.clone()])
            .collect()
    }

    #[test]
    fn boundary_classification_follows_frozen_semantics() {
        assert!(is_missing_space_boundary('用', 'L'));
        assert!(is_missing_space_boundary('M', '进'));
        assert!(is_missing_space_boundary('共', '1'));
        assert!(is_missing_space_boundary('0', '个'));
        assert!(!is_missing_space_boundary('3', 'c'), "Latin-digit pair");
        assert!(!is_missing_space_boundary('用', '。'), "CJK punctuation");
        assert!(!is_missing_space_boundary('（', 'L'), "CJK punctuation");
        assert!(!is_missing_space_boundary('M', ')'), "ASCII punctuation");
        assert!(!is_missing_space_boundary('用', '文'), "CJK-CJK");
        assert!(!is_missing_space_boundary('用', '　'), "whitespace char");
    }

    #[test]
    fn contiguous_mapping_reports_both_boundaries() {
        let text = "使用LLM推理".to_string();
        let len = text.len();
        let block = TextBlock {
            text,
            mappings: vec![mapping(0..len, 0..len)],
        };
        let findings = block_findings(&block, &[]);
        assert_eq!(slices(&block, &findings), ["用L", "M推"]);
    }

    #[test]
    fn split_mappings_with_source_gap_skip_the_dropped_boundary() {
        // Logical text "使用RPython实现" where the parser dropped source bytes
        // 7..21 ("/\allowbreak{}"), splitting the mappings.
        let text = "使用RPython实现".to_string();
        let len = text.len();
        let block = TextBlock {
            text,
            mappings: vec![mapping(0..7, 0..7), mapping(7..len, 21..21 + len - 7)],
        };
        let findings = block_findings(&block, &[]);
        assert_eq!(slices(&block, &findings), ["用R", "n实"]);
    }

    #[test]
    fn split_mappings_with_contiguous_source_keep_the_boundary() {
        let text = "中X英".to_string();
        let len = text.len();
        let block = TextBlock {
            text,
            mappings: vec![
                mapping(0..3, 0..3),
                mapping(3..4, 3..4),
                mapping(4..len, 4..len),
            ],
        };
        let findings = block_findings(&block, &[]);
        assert_eq!(slices(&block, &findings), ["中X", "X英"]);
    }

    #[test]
    fn whitespace_character_boundary_never_triggers() {
        let text = "使用 LLM".to_string();
        let len = text.len();
        let block = TextBlock {
            text,
            mappings: vec![mapping(0..len, 0..len)],
        };
        assert!(block_findings(&block, &[]).is_empty());
    }

    #[test]
    fn invalid_ignore_patterns_are_dropped_without_panicking() {
        let patterns = compile_ignore_patterns(&[
            "LLM".to_string(),
            "([a-z".to_string(),
            "*bad".to_string(),
            "(?P<".to_string(),
        ]);
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn exemption_overlap_silences_boundary_while_unrelated_boundaries_report() {
        let text = "第3章。方向1A".to_string();
        let len = text.len();
        let block = TextBlock {
            text,
            mappings: vec![mapping(0..len, 0..len)],
        };
        let patterns = compile_ignore_patterns(&["第\\d+章".to_string()]);
        let findings = block_findings(&block, &patterns);
        assert_eq!(slices(&block, &findings), ["向1"]);
    }
}
