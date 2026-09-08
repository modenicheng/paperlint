use crate::{
    config::PaperlintConfig,
    latex::{parser::Document, span::Span},
    lint::analysis::DocumentAnalysis,
    text::lexicon::Lexicon,
};
use std::ops::Range;

pub use crate::latex::span::{LocatedRange, ReadingPosition};
pub use crate::lint::terms::{
    AcronymDefinitionEntry, AcronymUsageEntry, DocumentTermRegistry, LexiconOccurrence,
};

/// Everything a rule needs for one lint run. Built once per document by the
/// engine and shared read-only across all rules.
pub struct LintContext<'a> {
    document: &'a Document,
    config: &'a PaperlintConfig,
    lexicon: &'a Lexicon,
    registry: DocumentTermRegistry,
    analysis: DocumentAnalysis,
}

impl<'a> LintContext<'a> {
    pub fn new(document: &'a Document, config: &'a PaperlintConfig, lexicon: &'a Lexicon) -> Self {
        let analysis = DocumentAnalysis::new(document, config, lexicon);
        let registry = DocumentTermRegistry::with_analysis(document, lexicon, &analysis);
        Self {
            document,
            config,
            lexicon,
            registry,
            analysis,
        }
    }

    pub const fn document(&self) -> &'a Document {
        self.document
    }

    pub const fn config(&self) -> &'a PaperlintConfig {
        self.config
    }

    pub const fn lexicon(&self) -> &'a Lexicon {
        self.lexicon
    }

    pub const fn registry(&self) -> &DocumentTermRegistry {
        &self.registry
    }

    pub const fn analysis(&self) -> &DocumentAnalysis {
        &self.analysis
    }

    /// Map a block-relative logical range to its LaTeX source span.
    pub fn span(&self, block: usize, range: Range<usize>) -> Option<Span> {
        let block = self.document.blocks.get(block)?;
        self.document.source_span(block, range)
    }

    /// Map a located range to its LaTeX source span.
    pub fn span_of(&self, located: &LocatedRange) -> Option<Span> {
        self.span(located.block, located.range.clone())
    }
}
