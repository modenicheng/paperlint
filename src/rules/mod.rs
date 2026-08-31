use crate::{
    config::PaperlintConfig,
    latex::{parser::Document, span::Span},
    lint::diagnostic::Diagnostic,
    rule_id::RuleId,
};
use regex::Regex;
use std::collections::HashMap;

pub trait Rule {
    fn id(&self) -> RuleId;
    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic>;
}

fn base_span(document: &Document) -> Span {
    Span {
        file: document.path.clone(),
        start: 0,
        end: document.text.len(),
        line: 1,
        column: 1,
    }
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
        let acronym = match Regex::new(r"\b[A-Z]{2,}\b") {
            Ok(regex) => regex,
            Err(_) => return Vec::new(),
        };
        acronym
            .find_iter(&document.text)
            .filter(|m| !rule.ignore.iter().any(|value| value == m.as_str()))
            .map(|m| Diagnostic {
                rule: self.id(),
                severity: rule.level,
                message: format!("acronym `{}` used before definition", m.as_str()),
                span: base_span(document),
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
        let acronym = match Regex::new(r"\b([A-Z]{2,})\b") {
            Ok(regex) => regex,
            Err(_) => return Vec::new(),
        };
        let mut counts: HashMap<String, usize> = HashMap::new();
        for capture in acronym.captures_iter(&document.text) {
            let key = capture[1].to_string();
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .filter(|(_, count)| *count < rule.min_occurrences)
            .map(|(acronym, _)| Diagnostic {
                rule: self.id(),
                severity: rule.level,
                message: format!("acronym `{}` is only used once", acronym),
                span: base_span(document),
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
        rule.replace
            .iter()
            .filter(|(wrong, _)| document.text.contains(*wrong))
            .map(|(wrong, right)| Diagnostic {
                rule: self.id(),
                severity: rule.level,
                message: format!("use `{}` instead of `{}`", right, wrong),
                span: base_span(document),
            })
            .collect()
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
        let sentence = match Regex::new(r"[^.!?]+[.!?]") {
            Ok(regex) => regex,
            Err(_) => return Vec::new(),
        };
        for m in sentence.find_iter(&document.text) {
            let word_count = m.as_str().split_whitespace().count();
            if word_count > rule.max_words {
                diagnostics.push(Diagnostic {
                    rule: self.id(),
                    severity: rule.level,
                    message: format!(
                        "sentence has {} words; max is {}",
                        word_count, rule.max_words
                    ),
                    span: base_span(document),
                });
            }
        }
        diagnostics
    }
}
