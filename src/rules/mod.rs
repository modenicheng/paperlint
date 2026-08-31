use crate::{
    config::PaperlintConfig,
    latex::parser::Document,
    lint::diagnostic::Diagnostic,
    rule_id::RuleId,
    text::chars::effective_length,
    text::{
        Language, detect_language, segment_sentences,
        terminology::{
            AcronymDefinition, AcronymUsage, extract_acronym_definitions, find_acronym_usages,
        },
    },
};
use std::{collections::HashMap, ops::Range};

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
        let analysis = analyze_acronyms(document);
        let first_definitions: HashMap<&str, ReadingPosition> = analysis
            .definitions
            .iter()
            .map(|definition| (definition.acronym.as_str(), definition.position()))
            .fold(HashMap::new(), |mut first, (acronym, position)| {
                first
                    .entry(acronym)
                    .and_modify(|existing| *existing = (*existing).min(position))
                    .or_insert(position);
                first
            });

        analysis
            .usages
            .iter()
            .filter(|usage| !usage.is_definition)
            .filter(|usage| {
                usage.acronym.len() >= rule.min_length
                    && !rule.ignore.iter().any(|value| value == &usage.acronym)
            })
            .filter(|usage| {
                first_definitions
                    .get(usage.acronym.as_str())
                    .is_none_or(|definition| usage.position() < *definition)
            })
            .filter_map(|usage| {
                let block = &document.blocks[usage.block];
                let span = document.source_span(block, usage.range.clone())?;
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

    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let rule = &config.rules.acr002;
        if !rule.level.is_enabled() {
            return Vec::new();
        }
        let analysis = analyze_acronyms(document);
        let mut first_definitions: HashMap<&str, &LocatedDefinition> = HashMap::new();
        for definition in &analysis.definitions {
            first_definitions
                .entry(definition.acronym.as_str())
                .and_modify(|existing| {
                    if definition.position() < existing.position() {
                        *existing = definition;
                    }
                })
                .or_insert(definition);
        }

        let mut findings: Vec<_> = first_definitions
            .into_iter()
            .filter_map(|(acronym, definition)| {
                let usage_count = analysis
                    .usages
                    .iter()
                    .filter(|usage| {
                        !usage.is_definition
                            && usage.acronym == acronym
                            && usage.position() > definition.position()
                    })
                    .count();
                (usage_count < rule.min_usages_after_definition)
                    .then_some((definition, usage_count))
            })
            .collect();
        findings.sort_by_key(|(definition, _)| definition.position());

        findings
            .into_iter()
            .filter_map(|(definition, usage_count)| {
                let block = &document.blocks[definition.block];
                let span = document.source_span(block, definition.range.clone())?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ReadingPosition {
    block: usize,
    byte: usize,
}

#[derive(Debug)]
struct LocatedDefinition {
    acronym: String,
    block: usize,
    range: Range<usize>,
}

impl LocatedDefinition {
    fn position(&self) -> ReadingPosition {
        ReadingPosition {
            block: self.block,
            byte: self.range.start,
        }
    }
}

#[derive(Debug)]
struct LocatedUsage {
    acronym: String,
    block: usize,
    range: Range<usize>,
    is_definition: bool,
}

impl LocatedUsage {
    fn position(&self) -> ReadingPosition {
        ReadingPosition {
            block: self.block,
            byte: self.range.start,
        }
    }
}

#[derive(Debug, Default)]
struct AcronymAnalysis {
    definitions: Vec<LocatedDefinition>,
    usages: Vec<LocatedUsage>,
}

fn analyze_acronyms(document: &Document) -> AcronymAnalysis {
    let mut analysis = AcronymAnalysis::default();
    for (block_index, block) in document.blocks.iter().enumerate() {
        let definitions = extract_acronym_definitions(&block.text);
        let definition_ranges: Vec<(String, Range<usize>)> = definitions
            .iter()
            .map(|definition: &AcronymDefinition| {
                (definition.acronym.clone(), definition.range.clone())
            })
            .collect();

        analysis.definitions.extend(definitions.into_iter().map(
            |definition: AcronymDefinition| LocatedDefinition {
                acronym: definition.acronym,
                block: block_index,
                range: definition.range,
            },
        ));
        analysis
            .usages
            .extend(
                find_acronym_usages(&block.text)
                    .into_iter()
                    .map(|usage: AcronymUsage| {
                        let is_definition = definition_ranges.iter().any(|(acronym, range)| {
                            acronym == &usage.acronym && *range == usage.range
                        });
                        LocatedUsage {
                            acronym: usage.acronym,
                            block: block_index,
                            range: usage.range,
                            is_definition,
                        }
                    }),
            );
    }
    analysis
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
