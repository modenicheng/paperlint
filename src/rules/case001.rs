//! CASE001: canonical casing for known acronyms, proper nouns, and units.
//!
//! The rule walks every block's logical text as `[A-Za-z][A-Za-z0-9]*`
//! tokens and reports a token only when it is a case variant of a canonical
//! form owned by the shared lexicon or by a document acronym definition.
//! Unknown mixed-case words stay silent, and fully lowercase variants of
//! all-uppercase canonicals stay silent so ordinary words that merely share
//! letters (e.g. `rag` vs `RAG`) are never flagged. TERM001 skips the
//! case-only replacements this rule already owns via
//! [`lexicon_case_covered`], so one wrong surface is never double-reported.

use crate::{
    lint::{context::LintContext, diagnostic::Diagnostic},
    rule_id::RuleId,
    text::lexicon::{Lexeme, LexemeKind, Lexicon},
};
use regex::Regex;
use std::{collections::HashMap, ops::Range, sync::OnceLock};

use super::Rule;

pub struct Case001;

impl Rule for Case001 {
    fn id(&self) -> RuleId {
        RuleId::Case001
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.case001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }

        // Reportable lexicon canonicals indexed by ASCII-folded letters.
        let mut canonical_by_letters: HashMap<String, String> = HashMap::new();
        for lexeme in context.lexicon().entries() {
            if case_eligible(lexeme) {
                canonical_by_letters
                    .entry(lexeme.canonical.to_ascii_lowercase())
                    .or_insert_with(|| lexeme.canonical.clone());
            }
        }
        // Document acronym definitions act as all-uppercase canonicals.
        let definitions: Vec<String> = context
            .registry()
            .definitions()
            .iter()
            .map(|definition| definition.acronym.clone())
            .collect();

        let document = context.document();
        let mut diagnostics = Vec::new();
        for (block_index, block) in document.blocks.iter().enumerate() {
            for range in ascii_tokens(&block.text) {
                let surface = &block.text[range.clone()];
                let Some(canonical) = canonical_by_letters
                    .get(&surface.to_ascii_lowercase())
                    .filter(|canonical| variant_reports(canonical.as_str(), surface))
                    .cloned()
                    .or_else(|| {
                        definitions
                            .iter()
                            .find(|acronym| acronym.eq_ignore_ascii_case(surface))
                            .filter(|acronym| variant_reports(acronym.as_str(), surface))
                            .cloned()
                    })
                else {
                    continue;
                };
                let Some(span) = context.span(block_index, range) else {
                    continue;
                };
                diagnostics.push(Diagnostic {
                    rule: self.id(),
                    severity: rule.level,
                    message: format!("use `{canonical}` instead of `{surface}`"),
                    span,
                });
            }
        }
        diagnostics
    }
}

/// Whether CASE001 would report `surface` against a lexicon canonical with
/// the same letters. TERM001 uses this to defer case-only replacements.
pub(crate) fn lexicon_case_covered(lexicon: &Lexicon, surface: &str) -> bool {
    is_ascii_token(surface)
        && lexicon.entries().any(|lexeme| {
            case_eligible(lexeme)
                && lexeme.canonical.eq_ignore_ascii_case(surface)
                && variant_reports(&lexeme.canonical, surface)
        })
}

/// Lexemes whose canonical casing CASE001 enforces: inherently
/// case-sensitive kinds the user has not opted out of, with a single ASCII
/// token canonical that carries at least one uppercase letter.
fn case_eligible(lexeme: &Lexeme) -> bool {
    matches!(
        lexeme.kind,
        LexemeKind::Acronym | LexemeKind::ProperNoun | LexemeKind::Unit | LexemeKind::Symbol
    ) && lexeme.case_sensitive
        && is_ascii_token(&lexeme.canonical)
        && lexeme.canonical.chars().any(|ch| ch.is_ascii_uppercase())
}

fn is_ascii_token(surface: &str) -> bool {
    let mut chars = surface.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric())
}

/// Whether `surface` is a reportable case variant of `canonical`.
///
/// Mixed-case canonicals (`GitHub`, `kHz`) own every other casing of their
/// letters. All-uppercase canonicals (`RAG`, `LLM`) report only mixed-case
/// surfaces: fully lowercase ones stay silent, and fully uppercase ones are
/// the canonical itself.
fn variant_reports(canonical: &str, surface: &str) -> bool {
    if surface == canonical {
        return false;
    }
    let has_upper = |text: &str| text.chars().any(|ch| ch.is_ascii_uppercase());
    let has_lower = |text: &str| text.chars().any(|ch| ch.is_ascii_lowercase());
    if has_upper(canonical) && has_lower(canonical) {
        true
    } else if has_upper(canonical) {
        has_upper(surface) && has_lower(surface)
    } else {
        false
    }
}

fn ascii_tokens(text: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    token_pattern()
        .find_iter(text)
        .map(|matched| matched.range())
}

fn token_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[A-Za-z][A-Za-z0-9]*").expect("valid regex"))
}
