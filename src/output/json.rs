use crate::lint::diagnostic::Diagnostic;

pub fn render(diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(diagnostics)
}
