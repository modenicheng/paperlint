use crate::{
    config::PaperlintConfig,
    latex::parser::Document,
    lint::{context::LintContext, diagnostic::Diagnostic, registry::RuleRegistry},
    text::lexicon::Lexicon,
};

pub struct RuleEngine {
    registry: RuleRegistry,
}

impl RuleEngine {
    pub fn new(registry: RuleRegistry) -> Self {
        Self { registry }
    }

    pub fn run(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let lexicon = Lexicon::from_config(config);
        let context = LintContext::new(document, config, &lexicon);
        let mut diagnostics = Vec::new();
        for rule in self.registry.enabled_rules() {
            diagnostics.extend(rule.check(&context));
        }
        diagnostics
    }
}
