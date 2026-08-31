/// Detected language/script of text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Chinese,
    English,
    Mixed,
}

/// Detect primary language of text based on character distribution
pub fn detect_language(text: &str) -> Language {
    let mut cjk_count = 0;
    let mut latin_count = 0;

    for ch in text.chars() {
        if is_cjk(ch) {
            cjk_count += 1;
        } else if ch.is_ascii_alphabetic() {
            latin_count += 1;
        }
    }

    if cjk_count == 0 && latin_count == 0 {
        return Language::Mixed;
    }

    if cjk_count > latin_count {
        Language::Chinese
    } else if latin_count > cjk_count {
        Language::English
    } else {
        Language::Mixed
    }
}

/// Check if character is CJK (Chinese, Japanese, Korean)
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |  // CJK Unified Ideographs Extension A
        '\u{20000}'..='\u{2A6DF}' | // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' | // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}' | // CJK Unified Ideographs Extension D
        '\u{2B820}'..='\u{2CEAF}' | // CJK Unified Ideographs Extension E
        '\u{F900}'..='\u{FAFF}' |   // CJK Compatibility Ideographs
        '\u{2F800}'..='\u{2FA1F}'   // CJK Compatibility Ideographs Supplement
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_chinese() {
        assert_eq!(detect_language("这是中文文本"), Language::Chinese);
        assert_eq!(detect_language("大语言模型"), Language::Chinese);
    }

    #[test]
    fn test_detect_english() {
        assert_eq!(detect_language("This is English text"), Language::English);
        assert_eq!(detect_language("Large Language Model"), Language::English);
    }

    #[test]
    fn test_detect_mixed() {
        assert_eq!(detect_language("使用 LLM 进行推理"), Language::Chinese);
        assert_eq!(detect_language("LLM 大语言模型"), Language::Chinese);
    }
}
