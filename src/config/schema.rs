use crate::rule_id::RuleId;
use serde::Deserialize;

use super::level::resolve;
use super::{
    Acr001Config, Acr002Config, Case001Config, Func001Config, Func002Config, Level,
    LexiconEntryConfig, NlpConfig, Punc002Config, Style001Config, Style002Config, Syn001Config,
    Syn002Config, Syn003Config, Syn004Config, Syn005Config, Term001Config, Term002Config,
    raw::RawPaperlintConfig,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexiconConfig {
    #[serde(default)]
    pub entries: Vec<LexiconEntryConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LatexConfig {
    #[serde(default)]
    pub ignore_environments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaperlintConfig {
    pub rules: RulesConfig,
    pub latex: LatexConfig,
    pub nlp: NlpConfig,
    pub lexicon: LexiconConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulesConfig {
    pub acr001: Acr001Config,
    pub acr002: Acr002Config,
    pub term001: Term001Config,
    pub term002: Term002Config,
    pub style001: Style001Config,
    pub punc002: Punc002Config,
    pub case001: Case001Config,
    pub func001: Func001Config,
    pub func002: Func002Config,
    pub style002: Style002Config,
    pub syn001: Syn001Config,
    pub syn002: Syn002Config,
    pub syn003: Syn003Config,
    pub syn004: Syn004Config,
    pub syn005: Syn005Config,
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
            Ok(RuleId::Term002) => self.rules.term002.level = level,
            Ok(RuleId::Style001) => self.rules.style001.level = level,
            Ok(RuleId::Punc002) => self.rules.punc002.level = level,
            Ok(RuleId::Case001) => self.rules.case001.level = level,
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
        if let Some(rule) = raw.rules.term002 {
            config.rules.term002 = resolve(rule, config.rules.term002.clone());
        }
        if let Some(rule) = raw.rules.style001 {
            config.rules.style001 = resolve(rule, config.rules.style001.clone());
        }
        if let Some(rule) = raw.rules.punc002 {
            config.rules.punc002 = resolve(rule, config.rules.punc002.clone());
        }
        if let Some(rule) = raw.rules.case001 {
            config.rules.case001 = resolve(rule, config.rules.case001.clone());
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
        if let Some(lexicon) = raw.lexicon {
            config.lexicon = LexiconConfig {
                entries: lexicon.entries,
            };
        }
        config
    }
}
