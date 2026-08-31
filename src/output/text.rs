use crate::lint::diagnostic::Diagnostic;

pub fn render(diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{}[{}]: {}\n",
                severity_label(diagnostic.severity),
                diagnostic.rule,
                diagnostic.message
            )
        })
        .collect()
}

fn severity_label(level: crate::config::Level) -> &'static str {
    match level {
        crate::config::Level::Off => "off",
        crate::config::Level::Warning => "warning",
        crate::config::Level::Error => "error",
    }
}
