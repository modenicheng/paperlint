use crate::{
    config::PaperlintConfig,
    latex::parser::Document,
    lint::{diagnostic::Diagnostic, registry::RuleRegistry},
};

pub struct RuleEngine {
    registry: RuleRegistry,
}

impl RuleEngine {
    pub fn new(registry: RuleRegistry) -> Self {
        Self { registry }
    }

    pub fn run(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in self.registry.enabled_rules() {
            diagnostics.extend(rule.check(document, config));
        }
        diagnostics
    }
}
