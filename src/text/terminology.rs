use regex::Regex;
use std::{ops::Range, sync::OnceLock};

/// Extract acronym definitions from one logical text block.
///
/// Recognized forms:
/// 1. 中文名（English Full Name，ABC）
/// 2. 中文名（English Full Name, ABC）
/// 3. 中文名（ABC）
/// 4. English full name (ABC)
///
/// Definitions do not span logical text blocks. The acronym range points to the
/// acronym itself so diagnostics can map it back to the exact LaTeX source.
pub fn extract_acronym_definitions(text: &str) -> Vec<AcronymDefinition> {
    let mut definitions = Vec::new();

    for captures in expanded_definition_pattern().captures_iter(text) {
        let (Some(whole), Some(english), Some(acronym)) =
            (captures.get(0), captures.get(1), captures.get(2))
        else {
            continue;
        };
        definitions.push(AcronymDefinition {
            chinese: preceding_chinese_phrase(text, whole.start()),
            english: Some(english.as_str().trim().to_string()),
            acronym: acronym.as_str().to_string(),
            range: acronym.range(),
        });
    }

    for captures in short_definition_pattern().captures_iter(text) {
        let (Some(whole), Some(acronym)) = (captures.get(0), captures.get(1)) else {
            continue;
        };
        let chinese = preceding_chinese_phrase(text, whole.start());
        let english = preceding_english_full_form(text, whole.start(), acronym.as_str());
        if chinese.is_none() && english.is_none() {
            continue;
        }
        definitions.push(AcronymDefinition {
            chinese,
            english,
            acronym: acronym.as_str().to_string(),
            range: acronym.range(),
        });
    }

    definitions.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then(left.range.end.cmp(&right.range.end))
            .then(left.acronym.cmp(&right.acronym))
    });
    definitions.dedup_by(|left, right| left.acronym == right.acronym && left.range == right.range);
    definitions
}

/// Find every acronym token in one logical text block.
pub fn find_acronym_usages(text: &str) -> Vec<AcronymUsage> {
    acronym_pattern()
        .captures_iter(text)
        .filter_map(|captures| captures.get(1))
        .map(|matched| AcronymUsage {
            acronym: matched.as_str().to_string(),
            range: matched.range(),
        })
        .collect()
}

fn acronym_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"(?-u:\b)([A-Z]{2,})(?-u:\b)").expect("valid regex"))
}

fn expanded_definition_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r"[\(\u{ff08}]\s*([A-Za-z][A-Za-z\s-]*[A-Za-z])\s*[,，]\s*([A-Z]{2,})\s*[\)\u{ff09}]",
        )
        .expect("valid regex")
    })
}

fn short_definition_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"[\(\u{ff08}]\s*([A-Z]{2,})\s*[\)\u{ff09}]").expect("valid regex")
    })
}

fn preceding_chinese_phrase(text: &str, open: usize) -> Option<String> {
    let prefix = text.get(..open)?.trim_end();
    let start = prefix
        .char_indices()
        .rev()
        .find_map(|(index, ch)| is_phrase_boundary(ch).then_some(index + ch.len_utf8()))
        .unwrap_or(0);
    let phrase = prefix[start..].trim();
    phrase.chars().any(is_cjk).then(|| phrase.to_string())
}

fn preceding_english_full_form(text: &str, open: usize, acronym: &str) -> Option<String> {
    let prefix = text.get(..open)?.trim_end();
    let start = prefix
        .char_indices()
        .rev()
        .find_map(|(index, ch)| {
            (!ch.is_ascii_alphabetic() && ch != '-' && ch != ' ' && ch != '\t')
                .then_some(index + ch.len_utf8())
        })
        .unwrap_or(0);
    let phrase = prefix[start..].trim();
    let words: Vec<_> = english_word_pattern().find_iter(phrase).collect();
    if words.len() < 2 {
        return None;
    }

    for word_start in (0..words.len() - 1).rev() {
        let candidate = &phrase[words[word_start].start()..words.last()?.end()];
        if initials_match(candidate, acronym) {
            return Some(candidate.to_string());
        }
    }
    None
}

fn english_word_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[A-Za-z]+").expect("valid regex"))
}

fn initials_match(full_form: &str, acronym: &str) -> bool {
    let initials: String = english_word_pattern()
        .find_iter(full_form)
        .flat_map(|word| word_initials(word.as_str()))
        .collect();
    initials == acronym
}

fn word_initials(word: &str) -> Vec<char> {
    let uppercase: Vec<_> = word.chars().filter(char::is_ascii_uppercase).collect();
    if uppercase.len() > 1 && !word.chars().all(|ch| ch.is_ascii_uppercase()) {
        uppercase
    } else {
        word.chars()
            .next()
            .map(|ch| ch.to_ascii_uppercase())
            .into_iter()
            .collect()
    }
}

fn is_phrase_boundary(ch: char) -> bool {
    matches!(
        ch,
        '.' | '。' | '!' | '！' | '?' | '？' | ';' | '；' | ':' | '：' | '\n' | '\r'
    )
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcronymDefinition {
    pub chinese: Option<String>,
    pub english: Option<String>,
    pub acronym: String,
    pub range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcronymUsage {
    pub acronym: String,
    pub range: Range<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_chinese_definition_with_full_width_comma() {
        let text = "大语言模型（Large Language Model，LLM）是一种新型模型。";
        let definitions = extract_acronym_definitions(text);

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].chinese, Some("大语言模型".to_string()));
        assert_eq!(
            definitions[0].english,
            Some("Large Language Model".to_string())
        );
        assert_eq!(definitions[0].acronym, "LLM");
        assert_eq!(&text[definitions[0].range.clone()], "LLM");
    }

    #[test]
    fn extracts_hyphenated_definition_with_ascii_comma() {
        let text = "检索增强生成（Retrieval-Augmented Generation, RAG）方法。";
        let definitions = extract_acronym_definitions(text);

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].acronym, "RAG");
    }

    #[test]
    fn extracts_chinese_only_definition() {
        let text = "大语言模型（LLM）进行推理。";
        let definitions = extract_acronym_definitions(text);

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].chinese, Some("大语言模型".to_string()));
        assert_eq!(definitions[0].english, None);
        assert_eq!(definitions[0].acronym, "LLM");
    }

    #[test]
    fn extracts_title_case_and_lowercase_english_definitions() {
        for text in [
            "We use Large Language Model (LLM) for inference.",
            "We use large language model (LLM) for inference.",
        ] {
            let definitions = extract_acronym_definitions(text);
            assert_eq!(definitions.len(), 1, "{text}");
            assert_eq!(
                definitions[0].english,
                Some(text[7..27].to_string()),
                "{text}"
            );
            assert_eq!(definitions[0].acronym, "LLM");
        }
    }

    #[test]
    fn parenthesized_uppercase_text_without_a_full_form_is_not_a_definition() {
        assert!(extract_acronym_definitions("Results are shown in (NASA).").is_empty());
    }

    #[test]
    fn finds_ascii_acronyms_next_to_chinese_text() {
        let text = "使用LLM和RAG技术进行NLP任务。";
        let usages = find_acronym_usages(text);

        assert_eq!(
            usages
                .iter()
                .map(|usage| usage.acronym.as_str())
                .collect::<Vec<_>>(),
            ["LLM", "RAG", "NLP"]
        );
        for usage in usages {
            assert_eq!(&text[usage.range], usage.acronym);
        }
    }
}
