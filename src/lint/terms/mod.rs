//! Shared term registry built once per lint run.

mod acronyms;
pub mod candidates;
mod lexicon_occurrences;
#[cfg(test)]
mod registry_tests;

use crate::{
    latex::{parser::Document, span::ReadingPosition},
    lint::analysis::DocumentAnalysis,
    text::lexicon::Lexicon,
};
use std::collections::HashMap;

pub use acronyms::{AcronymDefinitionEntry, AcronymUsageEntry};
use acronyms::{collect_acronyms, first_definitions_in_reading_order};
pub use candidates::{CandidateScore, TermCandidate, TermCandidateEvidence, TermCandidateKindHint};
pub use lexicon_occurrences::LexiconOccurrence;
use lexicon_occurrences::{LexiconScanner, first_occurrence_indexes, sort_in_reading_order};

/// Shared term analysis for one document, built once per lint run.
#[derive(Debug)]
pub struct DocumentTermRegistry {
    definitions: Vec<AcronymDefinitionEntry>,
    usages: Vec<AcronymUsageEntry>,
    first_definitions: Vec<AcronymDefinitionEntry>,
    lexicon_occurrences: Vec<LexiconOccurrence>,
    first_lexicon_occurrences: HashMap<String, usize>,
    term_candidates: Vec<TermCandidate>,
}

impl DocumentTermRegistry {
    /// Build all term analyses with one default lexical-analysis pass.
    pub fn new(document: &Document, lexicon: &Lexicon) -> Self {
        let analysis = DocumentAnalysis::with_default_config(document, lexicon);
        Self::with_analysis(document, lexicon, &analysis)
    }

    /// Build all term analyses from lexical analysis already owned by the caller.
    pub fn with_analysis(
        document: &Document,
        lexicon: &Lexicon,
        analysis: &DocumentAnalysis,
    ) -> Self {
        let mut definitions = Vec::new();
        let mut usages = Vec::new();
        let mut lexicon_occurrences = Vec::new();
        let frozen_matchers = lexicon.matchers();
        let lexicon_scanner = LexiconScanner::new(lexicon, frozen_matchers.as_deref());

        for (block_index, block) in document.blocks.iter().enumerate() {
            let acronym_entries = collect_acronyms(&block.text, block_index);
            definitions.extend(acronym_entries.definitions);
            usages.extend(acronym_entries.usages);
            lexicon_occurrences.extend(lexicon_scanner.collect(&block.text, block_index));
        }

        sort_in_reading_order(&mut lexicon_occurrences);
        let first_lexicon_occurrences = first_occurrence_indexes(&lexicon_occurrences);
        let first_definitions = first_definitions_in_reading_order(&definitions);
        let term_candidates = candidates::collect(analysis, lexicon, &definitions);

        Self {
            definitions,
            usages,
            first_definitions,
            lexicon_occurrences,
            first_lexicon_occurrences,
            term_candidates,
        }
    }

    /// All acronym definitions in reading order.
    pub fn definitions(&self) -> &[AcronymDefinitionEntry] {
        &self.definitions
    }

    /// All acronym occurrences in reading order, definition occurrences
    /// included and flagged.
    pub fn usages(&self) -> &[AcronymUsageEntry] {
        &self.usages
    }

    /// The earliest definition of `acronym`, if any.
    pub fn first_definition(&self, acronym: &str) -> Option<&AcronymDefinitionEntry> {
        self.first_definitions
            .iter()
            .find(|definition| definition.acronym == acronym)
    }

    /// Earliest definition per acronym, in reading order.
    pub fn first_definitions(&self) -> &[AcronymDefinitionEntry] {
        &self.first_definitions
    }

    /// Non-definition occurrences of `acronym` strictly after `position`, in
    /// reading order.
    pub fn usages_after(
        &self,
        acronym: &str,
        position: ReadingPosition,
    ) -> impl Iterator<Item = &AcronymUsageEntry> {
        self.usages.iter().filter(move |usage| {
            !usage.is_definition && usage.acronym == acronym && usage.location.position() > position
        })
    }

    /// All lexicon-resolved occurrences in reading order.
    pub fn lexicon_occurrences(&self) -> &[LexiconOccurrence] {
        &self.lexicon_occurrences
    }

    /// Lexicon-resolved occurrences of one canonical form, in reading order.
    pub fn lexicon_occurrences_of(
        &self,
        canonical: &str,
    ) -> impl Iterator<Item = &LexiconOccurrence> {
        self.lexicon_occurrences
            .iter()
            .filter(move |occurrence| occurrence.canonical == canonical)
    }

    /// The first lexicon-resolved occurrence of `canonical` in reading order.
    pub fn first_lexicon_occurrence(&self, canonical: &str) -> Option<&LexiconOccurrence> {
        self.first_lexicon_occurrences
            .get(canonical)
            .map(|&index| &self.lexicon_occurrences[index])
    }

    /// Unknown term candidates in deterministic reading order.
    pub fn term_candidates(&self) -> &[TermCandidate] {
        &self.term_candidates
    }
}
