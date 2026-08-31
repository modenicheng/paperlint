use crate::{
    config::PaperlintConfig,
    latex::parser::Document,
    lint::diagnostic::Diagnostic,
    rule_id::RuleId,
    text::chars::effective_length,
    text::{Language, detect_language, segment_sentences},
};
use regex::Regex;
use std::collections::HashMap;

pub trait Rule {
    fn id(&self) -> RuleId;
    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic>;
}

pub struct Acr001;
pub struct Acr002;
pub struct Term001;
pub struct Style001;

impl Rule for Acr001 {
    fn id(&self) -> RuleId {
        RuleId::Acr001
    }

    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let rule = &config.rules.acr001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let acronym = match Regex::new(r"(?-u:\b)[A-Z]{2,}(?-u:\b)") {
            Ok(regex) => regex,
            Err(_) => return Vec::new(),
        };
        document
            .blocks
            .iter()
            .flat_map(|block| {
                acronym.find_iter(&block.text).filter_map(|matched| {
                    if matched.as_str().len() < rule.min_length
                        || rule.ignore.iter().any(|value| value == matched.as_str())
                    {
                        return None;
                    }
                    let span = document.source_span(block, matched.range())?;
                    Some(Diagnostic {
                        rule: self.id(),
                        severity: rule.level,
                        message: format!("acronym `{}` used before definition", matched.as_str()),
                        span,
                    })
                })
            })
            .collect()
    }
}

impl Rule for Acr002 {
    fn id(&self) -> RuleId {
        RuleId::Acr002
    }

    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let rule = &config.rules.acr002;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let acronym = match Regex::new(r"(?-u:\b)([A-Z]{2,})(?-u:\b)") {
            Ok(regex) => regex,
            Err(_) => return Vec::new(),
        };
        let mut occurrences = HashMap::new();
        for block in &document.blocks {
            for capture in acronym.captures_iter(&block.text) {
                let Some(matched) = capture.get(1) else {
                    continue;
                };
                let Some(span) = document.source_span(block, matched.range()) else {
                    continue;
                };
                let entry = occurrences
                    .entry(matched.as_str().to_string())
                    .or_insert((0usize, span));
                entry.0 += 1;
            }
        }
        let mut findings: Vec<_> = occurrences.into_iter().collect();
        findings.sort_by(|left, right| {
            left.1
                .1
                .file
                .cmp(&right.1.1.file)
                .then(left.1.1.start.cmp(&right.1.1.start))
                .then(left.0.cmp(&right.0))
        });
        findings
            .into_iter()
            .filter(|(_, (count, _))| *count < rule.min_occurrences)
            .map(|(acronym, (_, span))| Diagnostic {
                rule: self.id(),
                severity: rule.level,
                message: format!("acronym `{acronym}` is only used once"),
                span,
            })
            .collect()
    }
}

impl Rule for Term001 {
    fn id(&self) -> RuleId {
        RuleId::Term001
    }

    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let rule = &config.rules.term001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let mut replacements: Vec<_> = rule.replace.iter().collect();
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

    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let rule = &config.rules.style001;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
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
