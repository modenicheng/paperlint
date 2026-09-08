use crate::text::lexicon::LexemeKind;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Off,
    Warning,
    Error,
}

impl Level {
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::Off)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acr001Config {
    pub level: Level,
    pub min_length: usize,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acr002Config {
    pub level: Level,
    #[serde(alias = "min_occurrences")]
    pub min_usages_after_definition: usize,
}

/// TERM002: shared lexicon terms with `requires_explanation = true`
/// must be explained near their first use. Term selection lives only in
/// `[[lexicon.entries]]`; this config never carries a term list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term002Config {
    pub level: Level,
    #[serde(default = "default_term002_context_chars")]
    pub context_chars: usize,
}

const fn default_term002_context_chars() -> usize {
    100
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term001Config {
    pub level: Level,
    #[serde(default)]
    pub replace: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style001Config {
    pub level: Level,
    #[serde(default = "default_style001_max_chars")]
    pub max_chars: usize,
    #[serde(alias = "max_words")]
    pub max_english_words: usize,
}

const fn default_style001_max_chars() -> usize {
    80
}

/// CASE001 has no knobs today; a dedicated config struct keeps the
/// documented `[rules.CASE001]` section open to future options without a
/// breaking config change.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case001Config {
    pub level: Level,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Punc002Config {
    pub level: Level,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Func001Config {
    pub level: Level,
    pub max_ratio: f32,
    pub min_sentence_tokens: usize,
    #[serde(default)]
    pub words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Func002Config {
    pub level: Level,
    pub max_same_class_in_window: usize,
    pub window_tokens: usize,
    #[serde(default)]
    pub classes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style002Config {
    pub level: Level,
    pub max_occurrences_per_paragraph: usize,
    #[serde(default)]
    pub words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syn001Config {
    pub level: Level,
    pub max_per_paragraph: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syn002Config {
    pub level: Level,
    pub max_per_paragraph: usize,
    pub max_consecutive_sentences: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syn003Config {
    pub level: Level,
    pub max_prepositional_phrases_before_main_clause: usize,
    #[serde(default)]
    pub words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syn004Config {
    pub level: Level,
    pub max_connectives_per_sentence: usize,
    #[serde(default)]
    pub words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syn005Config {
    pub level: Level,
    pub max_modifier_tokens: usize,
    pub require_dependency: bool,
}

/// One `[[lexicon.entries]]` workspace entry.
///
/// Surfaces are trimmed while deserializing, and empty or whitespace-only
/// canonical forms or aliases are rejected as configuration errors: an
/// empty surface would otherwise match at every position during occurrence
/// collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconEntryConfig {
    pub canonical: String,
    pub aliases: Vec<String>,
    pub kind: LexemeKind,
    /// Defaults to the sensible value for `kind` when omitted.
    pub case_sensitive: Option<bool>,
    pub requires_explanation: bool,
}

impl<'de> Deserialize<'de> for LexiconEntryConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            canonical: String,
            #[serde(default)]
            aliases: Vec<String>,
            kind: LexemeKind,
            #[serde(default)]
            case_sensitive: Option<bool>,
            #[serde(default)]
            requires_explanation: bool,
        }

        let raw = Raw::deserialize(deserializer)?;
        let canonical = raw.canonical.trim().to_string();
        if canonical.is_empty() {
            return Err(serde::de::Error::custom(
                "lexicon.entries: `canonical` must not be empty or whitespace",
            ));
        }
        let mut aliases = Vec::with_capacity(raw.aliases.len());
        for alias in raw.aliases {
            let alias = alias.trim().to_string();
            if alias.is_empty() {
                return Err(serde::de::Error::custom(
                    "lexicon.entries: `aliases` must not contain empty or whitespace entries",
                ));
            }
            aliases.push(alias);
        }
        Ok(Self {
            canonical,
            aliases,
            kind: raw.kind,
            case_sensitive: raw.case_sensitive,
            requires_explanation: raw.requires_explanation,
        })
    }
}
