use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DefaultConfig;

impl DefaultConfig {
    pub fn load() -> PaperlintConfig {
        let mut replace = HashMap::new();
        replace.insert("Github".to_string(), "GitHub".to_string());
        replace.insert("data set".to_string(), "dataset".to_string());
        replace.insert("web site".to_string(), "website".to_string());

        PaperlintConfig {
            nlp: NlpConfig {
                tokenizer: "jieba".into(),
                pos: "jieba".into(),
                syntax: "none".into(),
            },
            rules: super::rules::RulesConfig {
                acr001: Acr001Config {
                    level: Level::Error,
                    min_length: 2,
                    ignore: vec!["AI".into(), "CPU".into(), "GPU".into(), "API".into()],
                },
                acr002: Acr002Config {
                    level: Level::Warning,
                    min_usages_after_definition: 1,
                },
                term001: Term001Config {
                    level: Level::Warning,
                    replace,
                },
                style001: Style001Config {
                    level: Level::Warning,
                    max_chars: 80,
                    max_english_words: 45,
                },
                func001: Func001Config {
                    level: Level::Warning,
                    max_ratio: 0.20,
                    min_sentence_tokens: 10,
                    words: vec!["的".into(), "了".into(), "在".into(), "进行".into()],
                },
                func002: Func002Config {
                    level: Level::Warning,
                    max_same_class_in_window: 3,
                    window_tokens: 12,
                    classes: HashMap::from([(
                        "connective".into(),
                        vec!["由于".into(), "因此".into(), "从而".into(), "进而".into()],
                    )]),
                },
                style002: Style002Config {
                    level: Level::Warning,
                    max_occurrences_per_paragraph: 3,
                    words: vec!["进行".into(), "开展".into(), "实现".into(), "完成".into()],
                },
                syn001: Syn001Config {
                    level: Level::Warning,
                    max_per_paragraph: 2,
                },
                syn002: Syn002Config {
                    level: Level::Warning,
                    max_per_paragraph: 2,
                    max_consecutive_sentences: 2,
                },
                syn003: Syn003Config {
                    level: Level::Warning,
                    max_prepositional_phrases_before_main_clause: 3,
                    words: vec![
                        "在".into(),
                        "基于".into(),
                        "通过".into(),
                        "针对".into(),
                        "对于".into(),
                        "根据".into(),
                    ],
                },
                syn004: Syn004Config {
                    level: Level::Warning,
                    max_connectives_per_sentence: 3,
                    words: vec![
                        "由于".into(),
                        "因此".into(),
                        "从而".into(),
                        "进而".into(),
                        "同时".into(),
                        "此外".into(),
                        "但是".into(),
                        "然而".into(),
                    ],
                },
                syn005: Syn005Config {
                    level: Level::Warning,
                    max_modifier_tokens: 10,
                    require_dependency: false,
                },
            },
            latex: LatexConfig {
                ignore_environments: vec![
                    "align".into(),
                    "align*".into(),
                    "equation".into(),
                    "equation*".into(),
                    "lstlisting".into(),
                    "minted".into(),
                    "verbatim".into(),
                ],
            },
            lexicon: super::rules::LexiconConfig {
                entries: Vec::new(),
            },
        }
    }
}
