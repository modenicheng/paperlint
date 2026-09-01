//! TERM002 — configured terms must be explained near their first use.
//!
//! The rule consumes only [`DocumentTermRegistry::lexicon_occurrences`]:
//! for each canonical form whose lexeme sets `requires_explanation` and
//! whose kind is not [`Common`](LexemeKind::Common), exactly the first
//! occurrence in reading order is inspected; the rule never rescans text
//! on its own. The first occurrence counts as explained when either
//!
//! 1. it participates in a structured acronym definition registered for
//!    its block — the acronym span itself, or the Chinese/English full
//!    form adjacent to the definition's parentheses or inner comma, or
//! 2. a high-confidence trigger is anchored to the occurrence inside a
//!    window of `context_chars` characters within the same block:
//!    Chinese `X是/是指/指的是/指/即/定义为/表示` (only whitespace or
//!    `，,:：` may sit between X and the trigger), `所谓X`, `称X为`;
//!    English `X refers to/is defined as/denotes/stands for`,
//!    `X (also known as`, `X, i.e.,`, `X:`.
//!
//! Explanations after the first use never cancel a missing first-use
//! explanation. Each unexplained canonical form yields exactly one
//! diagnostic on its first occurrence.

use crate::{
    lint::{
        context::{AcronymDefinitionEntry, LexiconOccurrence, LintContext},
        diagnostic::Diagnostic,
    },
    rule_id::RuleId,
    text::lexicon::LexemeKind,
};
use regex::Regex;
use std::{collections::HashSet, ops::Range, sync::OnceLock};

pub struct Term002;

impl crate::rules::Rule for Term002 {
    fn id(&self) -> RuleId {
        RuleId::Term002
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.term002;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let registry = context.registry();
        let mut seen_canonicals: HashSet<&str> = HashSet::new();
        let mut diagnostics = Vec::new();
        for occurrence in registry.lexicon_occurrences() {
            if !occurrence.requires_explanation || occurrence.kind == LexemeKind::Common {
                continue;
            }
            if !seen_canonicals.insert(occurrence.canonical.as_str()) {
                continue;
            }
            if first_occurrence_is_explained(context, occurrence, rule.context_chars) {
                continue;
            }
            let Some(span) =
                context.span(occurrence.location.block, occurrence.location.range.clone())
            else {
                continue;
            };
            diagnostics.push(Diagnostic {
                rule: self.id(),
                severity: rule.level,
                message: format!(
                    "technical term `{}` first used without a nearby explanation",
                    occurrence.canonical
                ),
                span,
            });
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

/// Whether the first occurrence is explained by a structured declaration
/// or by an anchored trigger within `context_chars` characters.
fn first_occurrence_is_explained(
    context: &LintContext,
    occurrence: &LexiconOccurrence,
    context_chars: usize,
) -> bool {
    let Some(block) = context.document().blocks.get(occurrence.location.block) else {
        return false;
    };
    let text = block.text.as_str();
    if structured_definition_explains(context, occurrence, text) {
        return true;
    }
    let (window_start, window_end) = window_bounds(text, &occurrence.location.range, context_chars);
    let before = &text[window_start..occurrence.location.range.start];
    let after = &text[occurrence.location.range.end..window_end];
    forward_trigger().is_match(after)
        || preceding_trigger().is_match(before)
        || (calling_prefix().is_match(before) && called_suffix().is_match(after))
}

/// Structured acronym definitions: the occurrence is the definition's
/// acronym span, or sits in the same block directly before the definition
/// parentheses (Chinese/short full form) or before the inner comma
/// (English full form), and the declared phrase contains the surface.
fn structured_definition_explains(
    context: &LintContext,
    occurrence: &LexiconOccurrence,
    text: &str,
) -> bool {
    context
        .registry()
        .definitions()
        .iter()
        .any(|definition| structured_definition_matches(definition, occurrence, text))
}

fn structured_definition_matches(
    definition: &AcronymDefinitionEntry,
    occurrence: &LexiconOccurrence,
    text: &str,
) -> bool {
    if definition.location.block != occurrence.location.block {
        return false;
    }
    if occurrence.location.range == definition.location.range {
        return true;
    }
    if occurrence.location.range.end > definition.location.range.start {
        return false;
    }
    let Some(gap) = text.get(occurrence.location.range.end..definition.location.range.start) else {
        return false;
    };
    let trimmed = gap.trim_start();
    if trimmed.starts_with('(') || trimmed.starts_with('（') {
        return declares_surface(definition, &occurrence.surface);
    }
    if trimmed.starts_with(',') || trimmed.starts_with('，') {
        return definition
            .english
            .as_deref()
            .is_some_and(|phrase| phrase.contains(occurrence.surface.as_str()));
    }
    false
}

fn declares_surface(definition: &AcronymDefinitionEntry, surface: &str) -> bool {
    definition
        .chinese
        .as_deref()
        .is_some_and(|phrase| phrase.contains(surface))
        || definition
            .english
            .as_deref()
            .is_some_and(|phrase| phrase.contains(surface))
}

/// Trigger anchored right after the occurrence: only whitespace and
/// `，,:：` may separate X from the trigger phrase.
fn forward_trigger() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r"^[\s，,:：]*(?:是指|指的是|定义为|表示|即|指|是|(?i:refers\s+to|is\s+defined\s+as|denotes|stands\s+for|\(\s*also\s+known\s+as|i\.e\.,)|[:：])",
        )
        .expect("valid forward trigger regex")
    })
}

/// `所谓X` — the marker sits immediately before the occurrence.
fn preceding_trigger() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"所谓\s*\z").expect("valid preceding trigger regex"))
}

/// `称X为` — the calling marker before the occurrence plus 为 after it.
fn calling_prefix() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"称\s*\z").expect("valid calling prefix regex"))
}

fn called_suffix() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^\s*为").expect("valid called suffix regex"))
}

/// Byte offsets of the character window of `context_chars` characters on
/// each side of `range`, clamped to the block text.
fn window_bounds(text: &str, range: &Range<usize>, context_chars: usize) -> (usize, usize) {
    (
        backward_char_boundary(text, range.start, context_chars),
        forward_char_boundary(text, range.end, context_chars),
    )
}

fn backward_char_boundary(text: &str, from: usize, chars: usize) -> usize {
    let mut byte = from.min(text.len());
    let mut remaining = chars;
    while remaining > 0 && byte > 0 {
        byte -= 1;
        while byte > 0 && !text.is_char_boundary(byte) {
            byte -= 1;
        }
        remaining -= 1;
    }
    byte
}

fn forward_char_boundary(text: &str, from: usize, chars: usize) -> usize {
    let mut byte = from.min(text.len());
    let mut remaining = chars;
    while remaining > 0 && byte < text.len() {
        byte += 1;
        while byte < text.len() && !text.is_char_boundary(byte) {
            byte += 1;
        }
        remaining -= 1;
    }
    byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_bounds_clamp_to_block_and_char_boundaries() {
        let text = "使用智能体完成实验。";
        // 智能体 occupies bytes 6..15.
        assert_eq!(window_bounds(text, &(6..15), 2), (0, 21));
        assert_eq!(window_bounds(text, &(6..15), 0), (6, 15));
        let english = "we use attention here";
        // attention occupies bytes 7..16.
        assert_eq!(window_bounds(english, &(7..16), 3), (4, 19));
    }

    #[test]
    fn forward_trigger_accepts_frozen_phrases_and_rejects_possessives() {
        let trigger = forward_trigger();
        assert!(trigger.is_match("是指自主实体。"));
        assert!(trigger.is_match("是一种实体。"));
        assert!(trigger.is_match("，即自主实体。"));
        assert!(trigger.is_match("定义为自主实体。"));
        assert!(trigger.is_match("表示自主实体。"));
        assert!(trigger.is_match(": a weighting mechanism."));
        assert!(trigger.is_match(" (also known as a model)."));
        assert!(trigger.is_match(", i.e., the weights."));
        assert!(trigger.is_match(" refers to weights."));
        assert!(trigger.is_match(" is defined as weights."));
        assert!(trigger.is_match(" denotes weights."));
        assert!(trigger.is_match(" stands for weights."));
        assert!(!trigger.is_match("的性能是衡量标准。"));
        assert!(!trigger.is_match("机制仍在发展中。"));
        assert!(!trigger.is_match(" improves accuracy."));
    }
}
