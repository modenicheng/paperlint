use crate::{
    config::{
        DefaultConfig, NlpPosBackend, NlpSyntaxBackend, NlpTokenizerBackend, PaperlintConfig,
    },
    latex::{
        parser::Document,
        span::{LocatedRange, TextBlock},
    },
    nlp::{JiebaAnalyzer, LexicalAnalyzer, Token},
    text::{Language, detect_language, lexicon::Lexicon, segment_sentences},
};
#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static ANALYSIS_CONSTRUCTIONS: Cell<usize> = const { Cell::new(0) };
    static DEFAULT_ANALYSIS_CONSTRUCTIONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Debug, Clone)]
pub struct DocumentAnalysis {
    paragraphs: Vec<ParagraphUnit>,
}

#[derive(Debug, Clone)]
pub struct ParagraphUnit {
    pub block: usize,
    pub location: LocatedRange,
    pub sentences: Vec<SentenceUnit>,
}

#[derive(Debug, Clone)]
pub struct SentenceUnit {
    pub text: String,
    pub language: Language,
    pub location: LocatedRange,
    pub tokens: Vec<Token>,
}

impl DocumentAnalysis {
    pub fn new(document: &Document, config: &PaperlintConfig, lexicon: &Lexicon) -> Self {
        #[cfg(test)]
        ANALYSIS_CONSTRUCTIONS.with(|count| count.set(count.get() + 1));
        match config.nlp.syntax {
            NlpSyntaxBackend::None => {}
        }
        let lexical_analyzer = match (config.nlp.tokenizer, config.nlp.pos) {
            (NlpTokenizerBackend::Jieba, NlpPosBackend::Jieba) => {
                JiebaAnalyzer::with_workspace_lexicon(lexicon)
            }
        };
        let paragraphs = document
            .blocks
            .iter()
            .enumerate()
            .map(|(block, text_block)| analyze_block(block, text_block, &lexical_analyzer))
            .collect();
        Self { paragraphs }
    }

    pub(crate) fn with_default_config(document: &Document, lexicon: &Lexicon) -> Self {
        #[cfg(test)]
        DEFAULT_ANALYSIS_CONSTRUCTIONS.with(|count| count.set(count.get() + 1));
        Self::new(document, &DefaultConfig::load(), lexicon)
    }

    pub fn paragraphs(&self) -> &[ParagraphUnit] {
        &self.paragraphs
    }

    #[cfg(test)]
    pub(crate) fn from_paragraphs_for_tests(paragraphs: Vec<ParagraphUnit>) -> Self {
        Self { paragraphs }
    }

    #[cfg(test)]
    fn reset_construction_counts() {
        ANALYSIS_CONSTRUCTIONS.with(|count| count.set(0));
        DEFAULT_ANALYSIS_CONSTRUCTIONS.with(|count| count.set(0));
    }

    #[cfg(test)]
    fn construction_counts() -> (usize, usize) {
        (
            ANALYSIS_CONSTRUCTIONS.with(Cell::get),
            DEFAULT_ANALYSIS_CONSTRUCTIONS.with(Cell::get),
        )
    }
}

fn analyze_block(
    block_index: usize,
    block: &TextBlock,
    lexical_analyzer: &impl LexicalAnalyzer,
) -> ParagraphUnit {
    let sentences = segment_sentences(&block.text)
        .into_iter()
        .map(|sentence| {
            let location = LocatedRange {
                block: block_index,
                range: sentence.range,
            };
            let analysis = lexical_analyzer.analyze(&sentence.text, location.clone());
            SentenceUnit {
                text: sentence.text,
                language: detect_language(&analysis.text),
                location,
                tokens: analysis.tokens,
            }
        })
        .collect();
    ParagraphUnit {
        block: block_index,
        location: LocatedRange {
            block: block_index,
            range: 0..block.text.len(),
        },
        sentences,
    }
}

#[cfg(test)]
mod construction_tests {
    use super::DocumentAnalysis;
    use crate::{
        config::DefaultConfig,
        latex::parser,
        lint::{context::LintContext, terms::DocumentTermRegistry},
        text::lexicon::Lexicon,
    };

    #[test]
    fn lint_context_constructs_analysis_once_without_default_path() {
        let config = DefaultConfig::load();
        let document =
            parser::parse_stdin("GraphRAG2用于实验。".to_string(), &config.latex).unwrap();
        let lexicon = Lexicon::from_config(&config);
        DocumentAnalysis::reset_construction_counts();

        let _context = LintContext::new(&document, &config, &lexicon);

        assert_eq!(DocumentAnalysis::construction_counts(), (1, 0));
    }

    #[test]
    fn standalone_registry_constructs_one_default_analysis() {
        let config = DefaultConfig::load();
        let document =
            parser::parse_stdin("GraphRAG2用于实验。".to_string(), &config.latex).unwrap();
        let lexicon = Lexicon::from_config(&config);
        DocumentAnalysis::reset_construction_counts();

        let _registry = DocumentTermRegistry::new(&document, &lexicon);

        assert_eq!(DocumentAnalysis::construction_counts(), (1, 1));
    }
}
