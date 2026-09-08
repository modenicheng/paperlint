use serde::Deserialize;

use super::{
    Acr001Config, Acr002Config, Case001Config, Func001Config, Func002Config, LexiconEntryConfig,
    NlpConfig, Punc002Config, Style001Config, Style002Config, Syn001Config, Syn002Config,
    Syn003Config, Syn004Config, Syn005Config, Term001Config, Term002Config, level::RuleSetting,
};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawPaperlintConfig {
    pub rules: RawRulesConfig,
    pub latex: RawLatexConfig,
    pub nlp: Option<NlpConfig>,
    pub lexicon: Option<RawLexiconConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawRulesConfig {
    #[serde(alias = "ACR001")]
    pub acr001: Option<RuleSetting<Acr001Config>>,
    #[serde(alias = "ACR002")]
    pub acr002: Option<RuleSetting<Acr002Config>>,
    #[serde(alias = "TERM001")]
    pub term001: Option<RuleSetting<Term001Config>>,
    #[serde(alias = "TERM002")]
    pub term002: Option<RuleSetting<Term002Config>>,
    #[serde(alias = "STYLE001")]
    pub style001: Option<RuleSetting<Style001Config>>,
    #[serde(alias = "PUNC002")]
    pub punc002: Option<RuleSetting<Punc002Config>>,
    #[serde(alias = "CASE001")]
    pub case001: Option<RuleSetting<Case001Config>>,
    #[serde(alias = "FUNC001")]
    pub func001: Option<RuleSetting<Func001Config>>,
    #[serde(alias = "FUNC002")]
    pub func002: Option<RuleSetting<Func002Config>>,
    #[serde(alias = "STYLE002")]
    pub style002: Option<RuleSetting<Style002Config>>,
    #[serde(alias = "SYN001")]
    pub syn001: Option<RuleSetting<Syn001Config>>,
    #[serde(alias = "SYN002")]
    pub syn002: Option<RuleSetting<Syn002Config>>,
    #[serde(alias = "SYN003")]
    pub syn003: Option<RuleSetting<Syn003Config>>,
    #[serde(alias = "SYN004")]
    pub syn004: Option<RuleSetting<Syn004Config>>,
    #[serde(alias = "SYN005")]
    pub syn005: Option<RuleSetting<Syn005Config>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawLatexConfig {
    pub ignore_environments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawLexiconConfig {
    #[serde(default)]
    pub entries: Vec<LexiconEntryConfig>,
}
