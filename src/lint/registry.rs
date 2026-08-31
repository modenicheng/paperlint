use crate::rules::{Acr001, Acr002, Rule, Style001, Term001};

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self {
            rules: vec![
                Box::new(Acr001),
                Box::new(Acr002),
                Box::new(Term001),
                Box::new(Style001),
            ],
        }
    }
}

impl RuleRegistry {
    pub fn enabled_rules(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.iter().map(|rule| rule.as_ref())
    }
}
