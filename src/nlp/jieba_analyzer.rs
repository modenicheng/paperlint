use super::analyzer::{LexicalAnalysis, LexicalAnalyzer, PosTag, Token};
use crate::latex::span::Span;
use jieba_rs::Jieba;

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
}

impl Default for JiebaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LexicalAnalyzer for JiebaAnalyzer {
    fn analyze(&self, text: &str, base_span: &Span) -> LexicalAnalysis {
        let mut tokens = Vec::new();
        let mut current_pos = 0;

        // Use jieba to segment and tag
        for word in self.jieba.tag(text, false) {
            let word_text = word.word;
            let word_len = word_text.len();

            // Find the position in original text
            if let Some(pos) = text[current_pos..].find(word_text) {
                let absolute_pos = current_pos + pos;

                tokens.push(Token {
                    text: word_text.to_string(),
                    pos: map_jieba_pos(word.tag),
                    span: Span {
                        file: base_span.file.clone(),
                        start: base_span.start + absolute_pos,
                        end: base_span.start + absolute_pos + word_len,
                        line: base_span.line,
                        column: base_span.column,
                    },
                });

                current_pos = absolute_pos + word_len;
            }
        }

        LexicalAnalysis {
            text: text.to_string(),
            tokens,
            span: base_span.clone(),
        }
    }
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
    use std::path::PathBuf;

    fn make_span() -> Span {
        Span {
            file: PathBuf::from("test.tex"),
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }

    #[test]
    fn test_analyze_chinese() {
        let analyzer = JiebaAnalyzer::new();
        let span = make_span();
        let analysis = analyzer.analyze("我们使用大语言模型", &span);

        assert!(!analysis.tokens.is_empty());
        assert_eq!(analysis.text, "我们使用大语言模型");
    }

    #[test]
    fn test_analyze_mixed() {
        let analyzer = JiebaAnalyzer::new();
        let span = make_span();
        let analysis = analyzer.analyze("使用 LLM 进行推理", &span);

        assert!(!analysis.tokens.is_empty());
    }
}
