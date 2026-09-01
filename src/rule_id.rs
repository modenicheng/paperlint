use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleId {
    Acr001,
    Acr002,
    Term001,
    Term002,
    Style001,
    Func001,
    Func002,
    Style002,
    Syn001,
    Syn002,
    Syn003,
    Syn004,
    Syn005,
    Case001,
    Punc002,
}

impl RuleId {
    pub const ALL: [Self; 15] = [
        Self::Acr001,
        Self::Acr002,
        Self::Term001,
        Self::Term002,
        Self::Style001,
        Self::Func001,
        Self::Func002,
        Self::Style002,
        Self::Syn001,
        Self::Syn002,
        Self::Syn003,
        Self::Syn004,
        Self::Syn005,
        Self::Case001,
        Self::Punc002,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Acr001 => "ACR001",
            Self::Acr002 => "ACR002",
            Self::Term001 => "TERM001",
            Self::Term002 => "TERM002",
            Self::Style001 => "STYLE001",
            Self::Func001 => "FUNC001",
            Self::Func002 => "FUNC002",
            Self::Style002 => "STYLE002",
            Self::Syn001 => "SYN001",
            Self::Syn002 => "SYN002",
            Self::Syn003 => "SYN003",
            Self::Syn004 => "SYN004",
            Self::Syn005 => "SYN005",
            Self::Case001 => "CASE001",
            Self::Punc002 => "PUNC002",
        }
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RuleId {
    type Err = ParseRuleIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ACR001" => Ok(Self::Acr001),
            "ACR002" => Ok(Self::Acr002),
            "TERM001" => Ok(Self::Term001),
            "TERM002" => Ok(Self::Term002),
            "STYLE001" => Ok(Self::Style001),
            "FUNC001" => Ok(Self::Func001),
            "FUNC002" => Ok(Self::Func002),
            "STYLE002" => Ok(Self::Style002),
            "SYN001" => Ok(Self::Syn001),
            "SYN002" => Ok(Self::Syn002),
            "SYN003" => Ok(Self::Syn003),
            "SYN004" => Ok(Self::Syn004),
            "SYN005" => Ok(Self::Syn005),
            "CASE001" => Ok(Self::Case001),
            "PUNC002" => Ok(Self::Punc002),
            _ => Err(ParseRuleIdError {
                value: value.to_string(),
            }),
        }
    }
}

impl Serialize for RuleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RuleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRuleIdError {
    value: String,
}

impl fmt::Display for ParseRuleIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown rule id `{}`", self.value)
    }
}

impl std::error::Error for ParseRuleIdError {}
