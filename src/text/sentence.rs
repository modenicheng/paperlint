use crate::latex::span::Span;

/// A sentence extracted from document
#[derive(Debug, Clone)]
pub struct Sentence {
    pub text: String,
    pub span: Span,
}

/// Segment text into sentences
///
/// Handles both Chinese and English sentence boundaries:
/// - Chinese: 。！？；
/// - English: . ! ? ;
///
/// Note: This is a simplified v0.1 implementation.
/// May have false positives with abbreviations, decimals, etc.
pub fn segment_sentences(text: &str, base_span: &Span) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut current_start = 0;

    for (pos, ch) in text.char_indices() {
        if is_sentence_boundary(ch) {
            // Include the boundary character in the sentence
            let end_pos = pos + ch.len_utf8();
            let sentence_text = text[current_start..end_pos].trim();

            if !sentence_text.is_empty() {
                sentences.push(Sentence {
                    text: sentence_text.to_string(),
                    span: Span {
                        file: base_span.file.clone(),
                        start: base_span.start + current_start,
                        end: base_span.start + end_pos,
                        line: base_span.line, // Simplified: inherit base line
                        column: base_span.column,
                    },
                });
            }

            current_start = end_pos;
        }
    }

    // Handle remaining text if any
    if current_start < text.len() {
        let sentence_text = text[current_start..].trim();
        if !sentence_text.is_empty() {
            sentences.push(Sentence {
                text: sentence_text.to_string(),
                span: Span {
                    file: base_span.file.clone(),
                    start: base_span.start + current_start,
                    end: base_span.start + text.len(),
                    line: base_span.line,
                    column: base_span.column,
                },
            });
        }
    }

    sentences
}

/// Check if character is a sentence boundary
fn is_sentence_boundary(ch: char) -> bool {
    matches!(ch, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';')
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
    fn test_chinese_sentences() {
        let text = "这是第一句。这是第二句。";
        let span = make_span();
        let sentences = segment_sentences(text, &span);

        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "这是第一句。");
        assert_eq!(sentences[1].text, "这是第二句。");
    }

    #[test]
    fn test_english_sentences() {
        let text = "This is first. This is second.";
        let span = make_span();
        let sentences = segment_sentences(text, &span);

        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "This is first.");
        assert_eq!(sentences[1].text, "This is second.");
    }

    #[test]
    fn test_mixed_punctuation() {
        let text = "中文句子。English sentence. 混合！";
        let span = make_span();
        let sentences = segment_sentences(text, &span);

        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0].text, "中文句子。");
        assert_eq!(sentences[1].text, "English sentence.");
        assert_eq!(sentences[2].text, "混合！");
    }
}
