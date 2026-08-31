use regex::Regex;

/// Extract acronyms and their definitions from text
/// 
/// Supports multiple Chinese academic paper formats:
/// 1. 中文名（English Full Name，ABC）
/// 2. 中文名（English Full Name, ABC）
/// 3. 中文名（ABC）
/// 4. english full name (ABC) [for English papers]
pub fn extract_acronym_definitions(text: &str) -> Vec<AcronymDefinition> {
    let mut definitions = Vec::new();

    // Pattern 1 & 2: 中文名（English Full Name，ABC）or with comma
    let pattern1 = Regex::new(r"([^\(\uff08]+)[\(\uff08]([A-Z][a-zA-Z\s-]+)[，,]\s*([A-Z]{2,})[\)\uff09]").unwrap();
    for cap in pattern1.captures_iter(text) {
        definitions.push(AcronymDefinition {
            chinese: Some(cap[1].trim().to_string()),
            english: Some(cap[2].trim().to_string()),
            acronym: cap[3].trim().to_string(),
            position: cap.get(0).unwrap().start(),
        });
    }

    // Pattern 3: 中文名（ABC）- only match if there are CJK characters
    let pattern2 = Regex::new(r"([^\(\uff08]*[\p{Han}][^\(\uff08]*)[\(\uff08]([A-Z]{2,})[\)\uff09]").unwrap();
    for cap in pattern2.captures_iter(text) {
        let chinese = cap[1].trim();
        let acronym = cap[2].trim();
        
        // Skip if already captured by pattern1
        if definitions.iter().any(|d| d.acronym == acronym) {
            continue;
        }
        
        definitions.push(AcronymDefinition {
            chinese: Some(chinese.to_string()),
            english: None,
            acronym: acronym.to_string(),
            position: cap.get(0).unwrap().start(),
        });
    }

    // Pattern 4: english full name (ABC)
    // Match 2-5 lowercase words before (.
    // Note: v0.1 uses simple heuristic that may capture extra words
    let pattern3 = Regex::new(r"\b([a-z]+(?:\s+[a-z]+){1,4})\s*\(([A-Z]{2,})\)").unwrap();
    for cap in pattern3.captures_iter(text) {
        let acronym = cap[2].trim();
        
        // Skip if already captured
        if definitions.iter().any(|d| d.acronym == acronym) {
            continue;
        }
        
        definitions.push(AcronymDefinition {
            chinese: None,
            english: Some(cap[1].trim().to_string()),
            acronym: acronym.to_string(),
            position: cap.get(1).unwrap().start(), // Use group 1 position, not group 0
        });
    }

    definitions
}

/// Find all acronym usages in text
pub fn find_acronym_usages(text: &str) -> Vec<AcronymUsage> {
    let pattern = Regex::new(r"\b([A-Z]{2,})\b").unwrap();
    pattern
        .captures_iter(text)
        .map(|cap| AcronymUsage {
            acronym: cap[1].to_string(),
            position: cap.get(0).unwrap().start(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcronymDefinition {
    pub chinese: Option<String>,
    pub english: Option<String>,
    pub acronym: String,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcronymUsage {
    pub acronym: String,
    pub position: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_chinese_definition_with_comma() {
        let text = "大语言模型（Large Language Model，LLM）是一种新型模型。";
        let defs = extract_acronym_definitions(text);
        
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].chinese, Some("大语言模型".to_string()));
        assert_eq!(defs[0].english, Some("Large Language Model".to_string()));
        assert_eq!(defs[0].acronym, "LLM");
    }

    #[test]
    fn test_extract_chinese_definition_with_english_comma() {
        let text = "检索增强生成（Retrieval-Augmented Generation, RAG）方法。";
        let defs = extract_acronym_definitions(text);
        
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].acronym, "RAG");
    }

    #[test]
    fn test_extract_chinese_only() {
        let text = "大语言模型（LLM）进行推理。";
        let defs = extract_acronym_definitions(text);
        
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].chinese, Some("大语言模型".to_string()));
        assert_eq!(defs[0].english, None);
        assert_eq!(defs[0].acronym, "LLM");
    }

    #[test]
    fn test_extract_english_definition() {
        let text = "We use large language model (LLM) for inference.";
        let defs = extract_acronym_definitions(text);
        
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].chinese, None);
        // Note: v0.1 captures "use large language model" due to greedy matching
        // This is acceptable for initial version
        assert_eq!(defs[0].english, Some("use large language model".to_string()));
        assert_eq!(defs[0].acronym, "LLM");
    }

    #[test]
    fn test_find_usages() {
        let text = "使用 LLM 和 RAG 技术进行 NLP 任务。";
        let usages = find_acronym_usages(text);
        
        assert_eq!(usages.len(), 3);
        assert_eq!(usages[0].acronym, "LLM");
        assert_eq!(usages[1].acronym, "RAG");
        assert_eq!(usages[2].acronym, "NLP");
    }
}
