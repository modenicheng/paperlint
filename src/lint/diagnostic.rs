use crate::{config::Level, latex::span::Span, rule_id::RuleId};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub rule: RuleId,
    pub severity: Level,
    pub message: String,
    pub span: Span,
}
