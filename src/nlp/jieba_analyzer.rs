use super::analyzer::{LexicalAnalysis, LexicalAnalyzer, PosTag, Token};
use crate::{
    latex::span::LocatedRange,
    text::lexicon::{LexemeSource, Lexicon},
};
use jieba_rs::Jieba;
use std::collections::HashSet;

/// Jieba-based lexical analyzer for Chinese text
pub struct JiebaAnalyzer {
    jieba: Jieba,
}

impl JiebaAnalyzer {
    pub fn new() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }

    pub fn with_workspace_lexicon(lexicon: &Lexicon) -> Self {
        let mut analyzer = Self::new();
        for surface in workspace_surfaces(lexicon) {
            analyzer.jieba.add_word(surface, None, Some("nz"));
        }
        analyzer
    }
}

impl Default for JiebaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LexicalAnalyzer for JiebaAnalyzer {
    fn analyze(&self, text: &str, location: LocatedRange) -> LexicalAnalysis {
        let mut tokens = Vec::new();
        let mut current_pos = 0;

        for word in self.jieba.tag(text, false) {
            let word_text = word.word;
            let word_len = word_text.len();

            if let Some(pos) = text[current_pos..].find(word_text) {
                let absolute_pos = current_pos + pos;

                tokens.push(Token {
                    surface: word_text.to_string(),
                    pos: map_jieba_pos(word.tag),
                    location: LocatedRange {
                        block: location.block,
                        range: location.range.start + absolute_pos
                            ..location.range.start + absolute_pos + word_len,
                    },
                });

                current_pos = absolute_pos + word_len;
            }
        }

        LexicalAnalysis {
            text: text.to_string(),
            tokens,
            location,
        }
    }
}

fn workspace_surfaces(lexicon: &Lexicon) -> Vec<&str> {
    let mut seen = HashSet::new();
    let mut surfaces = Vec::new();
    for lexeme in lexicon
        .entries()
        .filter(|lexeme| lexeme.source == LexemeSource::Workspace)
    {
        for surface in std::iter::once(lexeme.canonical.as_str())
            .chain(lexeme.aliases.iter().map(String::as_str))
        {
            if surface.trim().is_empty() || surface.chars().any(char::is_whitespace) {
                continue;
            }
            if seen.insert(surface) {
                surfaces.push(surface);
            }
        }
    }
    surfaces
}

/// Map jieba POS tags to our PosTag enum
///
/// Jieba uses tags like: n (noun), v (verb), a (adj), etc.
/// Reference: https://github.com/fxsjy/jieba#%E8%AF%8D%E6%80%A7%E6%A0%87%E6%B3%A8
fn map_jieba_pos(tag: &str) -> PosTag {
    match tag {
        // Nouns
        "n" | "nr" | "ns" | "nt" | "nz" | "ng" => PosTag::Noun,

        // Verbs
        "v" | "vd" | "vn" | "vg" => PosTag::Verb,

        // Adjectives
        "a" | "ad" | "an" | "ag" => PosTag::Adjective,

        // Adverbs
        "d" => PosTag::Adverb,

        // Prepositions
        "p" => PosTag::Preposition,

        // Conjunctions
        "c" => PosTag::Conjunction,

        // Pronouns
        "r" => PosTag::Pronoun,

        // Particles (助词)
        "u" | "uz" | "ug" | "ul" | "uv" => PosTag::Particle,

        // Numerals
        "m" | "mq" => PosTag::Numeral,

        // Punctuation
        "x" | "w" => PosTag::Punctuation,

        // Others
        _ => PosTag::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(end: usize) -> LocatedRange {
        LocatedRange {
            block: 0,
            range: 0..end,
        }
    }

    #[test]
    fn test_analyze_chinese() {
        let analyzer = JiebaAnalyzer::new();
        let text = "我们使用大语言模型";
        let analysis = analyzer.analyze(text, location(text.len()));

        assert!(!analysis.tokens.is_empty());
        assert_eq!(analysis.text, "我们使用大语言模型");
    }

    #[test]
    fn test_analyze_mixed() {
        let analyzer = JiebaAnalyzer::new();
        let text = "使用 LLM 进行推理";
        let analysis = analyzer.analyze(text, location(text.len()));

        assert!(!analysis.tokens.is_empty());
    }
}
