use crate::{
    lint::{context::LintContext, diagnostic::Diagnostic},
    rule_id::RuleId,
    text::{
        Language, chars::effective_length, detect_language, lexicon::LexemeKind, segment_sentences,
    },
};

mod case001;
mod punc002;
mod term002;

pub use case001::Case001;
pub use punc002::Punc002;
pub use term002::Term002;

pub trait Rule {
    fn id(&self) -> RuleId;
    fn check(&self, context: &LintContext) -> Vec<Diagnostic>;
}

pub struct Acr001;
pub struct Acr002;
pub struct Term001;
pub struct Style001;

impl Rule for Acr001 {
    fn id(&self) -> RuleId {
        RuleId::Acr001
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.acr001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let registry = context.registry();
        // Denoising: an acronym the shared lexicon already knows (built-in
        // core, workspace entries, or `ACR001.ignore`) does not need a
        // document definition. `find_acronym_usages` yields uppercase
        // surfaces, so a plain lookup answers whether the lexicon owns it.
        let lexicon = context.lexicon();
        let lexicon_known = |acronym: &str| {
            lexicon.lookup(acronym).is_some_and(|lexeme| {
                matches!(
                    lexeme.kind,
                    LexemeKind::Acronym
                        | LexemeKind::ProperNoun
                        | LexemeKind::Unit
                        | LexemeKind::Symbol
                        | LexemeKind::Common
                )
            }) || crate::rules::case001::lexicon_case_covered(lexicon, acronym)
        };
        registry
            .usages()
            .iter()
            .filter(|usage| !usage.is_definition)
            .filter(|usage| {
                usage.acronym.len() >= rule.min_length
                    && !rule.ignore.iter().any(|value| value == &usage.acronym)
            })
            .filter(|usage| !lexicon_known(&usage.acronym))
            .filter(|usage| {
                registry
                    .first_definition(&usage.acronym)
                    .is_none_or(|definition| {
                        usage.location.position() < definition.location.position()
                    })
            })
            .filter_map(|usage| {
                let span = context.span(usage.location.block, usage.location.range.clone())?;
                Some(Diagnostic {
                    rule: self.id(),
                    severity: rule.level,
                    message: format!("acronym `{}` used before definition", usage.acronym),
                    span,
                })
            })
            .collect()
    }
}

impl Rule for Acr002 {
    fn id(&self) -> RuleId {
        RuleId::Acr002
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.acr002;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let registry = context.registry();
        registry
            .first_definitions()
            .iter()
            .filter_map(|definition| {
                let usage_count = registry
                    .usages_after(&definition.acronym, definition.location.position())
                    .count();
                (usage_count < rule.min_usages_after_definition)
                    .then_some((definition, usage_count))
            })
            .filter_map(|(definition, usage_count)| {
                let span = context.span(definition.location.block, definition.location.range.clone())?;
                let usage_word = if usage_count == 1 { "use" } else { "uses" };
                Some(Diagnostic {
                    rule: self.id(),
                    severity: rule.level,
                    message: format!(
                        "acronym `{}` has {usage_count} {usage_word} after its definition; minimum is {}",
                        definition.acronym, rule.min_usages_after_definition
                    ),
                    span,
                })
            })
            .collect()
    }
}

impl Rule for Term001 {
    fn id(&self) -> RuleId {
        RuleId::Term001
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.term001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let document = context.document();
        // CASE001 owns case-only corrections for lexicon canonicals (e.g.
        // `Github` -> `GitHub`); TERM001 defers to it so one wrong surface is
        // never reported by both rules.
        let lexicon = context.lexicon();
        let mut replacements: Vec<_> = rule
            .replace
            .iter()
            .filter(|(wrong, _)| !crate::rules::case001::lexicon_case_covered(lexicon, wrong))
            .collect();
        replacements.sort_by(|left, right| left.0.cmp(right.0));
        let mut diagnostics = Vec::new();
        for block in &document.blocks {
            for (wrong, right) in &replacements {
                for (start, _) in block.text.match_indices(wrong.as_str()) {
                    let end = start + wrong.len();
                    if let Some(span) = document.source_span(block, start..end) {
                        diagnostics.push(Diagnostic {
                            rule: self.id(),
                            severity: rule.level,
                            message: format!("use `{right}` instead of `{wrong}`"),
                            span,
                        });
                    }
                }
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

impl Rule for Style001 {
    fn id(&self) -> RuleId {
        RuleId::Style001
    }

    fn check(&self, context: &LintContext) -> Vec<Diagnostic> {
        let rule = &context.config().rules.style001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let document = context.document();
        let mut diagnostics = Vec::new();
        let logical_base = crate::text::sentence::logical_base_span();
        for block in &document.blocks {
            for sentence in segment_sentences(&block.text, &logical_base) {
                let language = detect_language(&sentence.text);
                let (length, max, unit) = match language {
                    Language::English => (
                        sentence.text.split_whitespace().count(),
                        rule.max_english_words,
                        "words",
                    ),
                    Language::Chinese | Language::Mixed => (
                        effective_length(&sentence.text, language),
                        rule.max_chars,
                        "effective characters",
                    ),
                };
                if length <= max {
                    continue;
                }
                if let Some(span) = document.source_span(block, sentence.range) {
                    diagnostics.push(Diagnostic {
                        rule: self.id(),
                        severity: rule.level,
                        message: format!("sentence has {length} {unit}; max is {max}"),
                        span,
                    });
                }
            }
        }
        diagnostics
    }
}
