use crate::lint::diagnostic::Diagnostic;

pub fn render(diagnostics: &[Diagnostic]) -> String {
    match serde_json::to_string_pretty(diagnostics) {
        Ok(output) => output,
        Err(_) => String::from("[]"),
    }
}
