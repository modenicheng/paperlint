use std::ops::Range;

/// A sentence extracted from logical document text.
#[derive(Debug, Clone)]
pub struct Sentence {
    pub text: String,
    /// Exact half-open UTF-8 byte range in the logical text passed to the segmenter.
    pub range: Range<usize>,
}

/// Segment text into Chinese and English sentences.
pub fn segment_sentences(text: &str) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut current_start = 0;

    for (pos, ch) in text.char_indices() {
        if is_sentence_boundary(ch) {
            let end = pos + ch.len_utf8();
            push_sentence(&mut sentences, text, current_start..end);
            current_start = end;
        }
    }
    if current_start < text.len() {
        push_sentence(&mut sentences, text, current_start..text.len());
    }
    sentences
}

fn push_sentence(sentences: &mut Vec<Sentence>, text: &str, candidate: Range<usize>) {
    let raw = &text[candidate.clone()];
    let trimmed = raw.trim();
    let leading = raw.find(trimmed).unwrap_or(0);
    let range = candidate.start + leading..candidate.start + leading + trimmed.len();
    if range.start == range.end {
        return;
    }
    sentences.push(Sentence {
        text: text[range.clone()].to_string(),
        range,
    });
}

fn is_sentence_boundary(ch: char) -> bool {
    matches!(ch, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_sentences() {
        let sentences = segment_sentences("这是第一句。这是第二句。");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "这是第一句。");
        assert_eq!(sentences[1].text, "这是第二句。");
    }

    #[test]
    fn test_english_sentences() {
        let sentences = segment_sentences("This is first. This is second.");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "This is first.");
        assert_eq!(sentences[1].text, "This is second.");
    }

    #[test]
    fn test_mixed_punctuation() {
        let sentences = segment_sentences("中文句子。English sentence. 混合！");
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0].text, "中文句子。");
        assert_eq!(sentences[1].text, "English sentence.");
        assert_eq!(sentences[2].text, "混合！");
    }

    #[test]
    fn range_accounts_for_trimmed_leading_and_trailing_whitespace() {
        let text = " \r\n  中文句子。  ";
        let sentences = segment_sentences(text);
        assert_eq!(
            sentences[0].range,
            text.find('中').unwrap()..text.find('。').unwrap() + 3
        );
        assert_eq!(&text[sentences[0].range.clone()], "中文句子。");
    }
}
