use serde::Deserialize;

use super::{
    Acr001Config, Acr002Config, Case001Config, Func001Config, Func002Config, Level, Punc002Config,
    Style001Config, Style002Config, Syn001Config, Syn002Config, Syn003Config, Syn004Config,
    Syn005Config, Term001Config, Term002Config,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleSetting<T> {
    Enabled(bool),
    Config(T),
}

pub(super) fn resolve<T: HasLevel>(setting: RuleSetting<T>, default: T) -> T {
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

pub(super) trait HasLevel {
    fn set_level(&mut self, level: Level);
}

macro_rules! impl_has_level {
    ($($config:ty),+ $(,)?) => {
        $(
            impl HasLevel for $config {
                fn set_level(&mut self, level: Level) {
                    self.level = level;
                }
            }
        )+
    };
}

impl_has_level!(
    Acr001Config,
    Acr002Config,
    Term001Config,
    Term002Config,
    Style001Config,
    Punc002Config,
    Case001Config,
    Func001Config,
    Func002Config,
    Style002Config,
    Syn001Config,
    Syn002Config,
    Syn003Config,
    Syn004Config,
    Syn005Config,
);
