use crate::rule_id::RuleId;
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
    pub min_occurrences: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LatexConfig {
    #[serde(default)]
    pub ignore_environments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NlpConfig {
    pub tokenizer: String,
    pub pos: String,
    pub syntax: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaperlintConfig {
    pub rules: RulesConfig,
    pub latex: LatexConfig,
    pub nlp: NlpConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulesConfig {
    pub acr001: Acr001Config,
    pub acr002: Acr002Config,
    pub term001: Term001Config,
    pub style001: Style001Config,
    pub func001: Func001Config,
    pub func002: Func002Config,
    pub style002: Style002Config,
    pub syn001: Syn001Config,
    pub syn002: Syn002Config,
    pub syn003: Syn003Config,
    pub syn004: Syn004Config,
    pub syn005: Syn005Config,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawPaperlintConfig {
    pub rules: RawRulesConfig,
    pub latex: RawLatexConfig,
    pub nlp: Option<NlpConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawRulesConfig {
    pub acr001: Option<RuleSetting<Acr001Config>>,
    pub acr002: Option<RuleSetting<Acr002Config>>,
    pub term001: Option<RuleSetting<Term001Config>>,
    #[serde(alias = "STYLE001")]
    pub style001: Option<RuleSetting<Style001Config>>,
    pub func001: Option<RuleSetting<Func001Config>>,
    pub func002: Option<RuleSetting<Func002Config>>,
    pub style002: Option<RuleSetting<Style002Config>>,
    pub syn001: Option<RuleSetting<Syn001Config>>,
    pub syn002: Option<RuleSetting<Syn002Config>>,
    pub syn003: Option<RuleSetting<Syn003Config>>,
    pub syn004: Option<RuleSetting<Syn004Config>>,
    pub syn005: Option<RuleSetting<Syn005Config>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawLatexConfig {
    pub ignore_environments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleSetting<T> {
    Enabled(bool),
    Config(T),
}

impl PaperlintConfig {
    pub fn with_overrides(mut self, enable: &[String], disable: &[String]) -> Self {
        for rule in enable {
            self = self.set_rule_level(rule, Level::Warning);
        }
        for rule in disable {
            self = self.set_rule_level(rule, Level::Off);
        }
        self
    }

    fn set_rule_level(mut self, rule: &str, level: Level) -> Self {
        match rule.parse::<RuleId>() {
            Ok(RuleId::Acr001) => self.rules.acr001.level = level,
            Ok(RuleId::Acr002) => self.rules.acr002.level = level,
            Ok(RuleId::Term001) => self.rules.term001.level = level,
            Ok(RuleId::Style001) => self.rules.style001.level = level,
            Ok(RuleId::Func001) => self.rules.func001.level = level,
            Ok(RuleId::Func002) => self.rules.func002.level = level,
            Ok(RuleId::Style002) => self.rules.style002.level = level,
            Ok(RuleId::Syn001) => self.rules.syn001.level = level,
            Ok(RuleId::Syn002) => self.rules.syn002.level = level,
            Ok(RuleId::Syn003) => self.rules.syn003.level = level,
            Ok(RuleId::Syn004) => self.rules.syn004.level = level,
            Ok(RuleId::Syn005) => self.rules.syn005.level = level,
            Err(_) => {}
        }
        self
    }

    pub fn from_raw(raw: RawPaperlintConfig, defaults: Self) -> Self {
        let mut config = defaults;
        if let Some(rule) = raw.rules.acr001 {
            config.rules.acr001 = resolve(rule, config.rules.acr001.clone());
        }
        if let Some(rule) = raw.rules.acr002 {
            config.rules.acr002 = resolve(rule, config.rules.acr002.clone());
        }
        if let Some(rule) = raw.rules.term001 {
            config.rules.term001 = resolve(rule, config.rules.term001.clone());
        }
        if let Some(rule) = raw.rules.style001 {
            config.rules.style001 = resolve(rule, config.rules.style001.clone());
        }
        if let Some(rule) = raw.rules.func001 {
            config.rules.func001 = resolve(rule, config.rules.func001.clone());
        }
        if let Some(rule) = raw.rules.func002 {
            config.rules.func002 = resolve(rule, config.rules.func002.clone());
        }
        if let Some(rule) = raw.rules.style002 {
            config.rules.style002 = resolve(rule, config.rules.style002.clone());
        }
        if let Some(rule) = raw.rules.syn001 {
            config.rules.syn001 = resolve(rule, config.rules.syn001.clone());
        }
        if let Some(rule) = raw.rules.syn002 {
            config.rules.syn002 = resolve(rule, config.rules.syn002.clone());
        }
        if let Some(rule) = raw.rules.syn003 {
            config.rules.syn003 = resolve(rule, config.rules.syn003.clone());
        }
        if let Some(rule) = raw.rules.syn004 {
            config.rules.syn004 = resolve(rule, config.rules.syn004.clone());
        }
        if let Some(rule) = raw.rules.syn005 {
            config.rules.syn005 = resolve(rule, config.rules.syn005.clone());
        }
        if let Some(ignore_environments) = raw.latex.ignore_environments {
            config.latex.ignore_environments = ignore_environments;
        }
        if let Some(nlp) = raw.nlp {
            config.nlp = nlp;
        }
        config
    }
}

fn resolve<T: HasLevel>(setting: RuleSetting<T>, default: T) -> T {
    match setting {
        RuleSetting::Enabled(true) => default,
        RuleSetting::Enabled(false) => {
            let mut config = default;
            config.set_level(Level::Off);
            config
        }
        RuleSetting::Config(value) => value,
    }
}

trait HasLevel {
    fn set_level(&mut self, level: Level);
}

impl HasLevel for Acr001Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Acr002Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Term001Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Style001Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Func001Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Func002Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Style002Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Syn001Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Syn002Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Syn003Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Syn004Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

impl HasLevel for Syn005Config {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}
