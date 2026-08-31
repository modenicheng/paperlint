use super::language::Language;

/// Statistics about sentence content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceStats {
    /// Number of CJK characters
    pub cjk_chars: usize,
    /// Number of Latin words (space-separated)
    pub latin_words: usize,
    /// Number of numeric tokens
    pub numeric_tokens: usize,
    /// Effective length = cjk_chars + latin_words + numeric_tokens
    pub effective_length: usize,
}

/// Calculate statistics for a sentence
pub fn calculate_stats(text: &str) -> SentenceStats {
    let mut cjk_chars = 0;
    let mut latin_words = 0;
    let mut numeric_tokens = 0;

    // Count CJK characters
    for ch in text.chars() {
        if is_cjk(ch) {
            cjk_chars += 1;
        }
    }

    // Count Latin words and numeric tokens
    // Split by whitespace and punctuation
    for token in text.split(|c: char| c.is_whitespace() || is_punctuation(c)) {
        if token.is_empty() {
            continue;
        }

        if token.chars().all(|c| c.is_ascii_alphabetic()) {
            latin_words += 1;
        } else if token.chars().all(|c| c.is_ascii_digit() || c == '.') {
            numeric_tokens += 1;
        }
    }

    let effective_length = cjk_chars + latin_words + numeric_tokens;

    SentenceStats {
        cjk_chars,
        latin_words,
        numeric_tokens,
        effective_length,
    }
}

/// Get effective length based on language
pub fn effective_length(text: &str, language: Language) -> usize {
    match language {
        Language::Chinese | Language::Mixed => calculate_stats(text).effective_length,
        Language::English => text.split_whitespace().count(),
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{20000}'..='\u{2A6DF}' |
        '\u{2A700}'..='\u{2B73F}' |
        '\u{2B740}'..='\u{2B81F}' |
        '\u{2B820}'..='\u{2CEAF}' |
        '\u{F900}'..='\u{FAFF}' |
        '\u{2F800}'..='\u{2FA1F}'
    )
}

fn is_punctuation(ch: char) -> bool {
    matches!(ch,
        '.' | ',' | '!' | '?' | ';' | ':' | 
        '。' | '，' | '！' | '？' | '；' | '：' |
        '(' | ')' | '（' | '）' | '[' | ']' | 
        '{' | '}' | '"' | '\'' |
        '\u{2018}' | '\u{2019}' // Left and right single quotation marks
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_stats() {
        let stats = calculate_stats("这是中文测试");
        assert_eq!(stats.cjk_chars, 6);
        assert_eq!(stats.latin_words, 0);
        assert_eq!(stats.effective_length, 6);
    }

    #[test]
    fn test_english_stats() {
        let stats = calculate_stats("This is a test");
        assert_eq!(stats.cjk_chars, 0);
        assert_eq!(stats.latin_words, 4);
        assert_eq!(stats.effective_length, 4);
    }

    #[test]
    fn test_mixed_stats() {
        let stats = calculate_stats("使用 LLM 进行推理");
        assert_eq!(stats.cjk_chars, 6);
        assert_eq!(stats.latin_words, 1); // LLM
        assert_eq!(stats.effective_length, 7);
    }

    #[test]
    fn test_with_numbers() {
        let stats = calculate_stats("共有 100 个样本");
        assert_eq!(stats.cjk_chars, 5);
        assert_eq!(stats.numeric_tokens, 1);
        assert_eq!(stats.effective_length, 6);
    }
}
