use crate::latex::span::LocatedRange;

/// Part-of-speech tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PosTag {
    /// Noun
    Noun,
    /// Verb
    Verb,
    /// Adjective
    Adjective,
    /// Adverb
    Adverb,
    /// Preposition
    Preposition,
    /// Conjunction
    Conjunction,
    /// Pronoun
    Pronoun,
    /// Particle (助词)
    Particle,
    /// Numeral
    Numeral,
    /// Punctuation
    Punctuation,
    /// Other/Unknown
    Other,
}

/// A token with position and POS tag
#[derive(Debug, Clone)]
pub struct Token {
    pub surface: String,
    pub pos: PosTag,
    pub location: LocatedRange,
}

/// Result of lexical analysis on a sentence
#[derive(Debug, Clone)]
pub struct LexicalAnalysis {
    pub text: String,
    pub tokens: Vec<Token>,
    pub location: LocatedRange,
}

/// Trait for lexical analyzers (tokenizer + POS tagger)
pub trait LexicalAnalyzer {
    /// Analyze a sentence into tokens with POS tags
    fn analyze(&self, text: &str, location: LocatedRange) -> LexicalAnalysis;
}
