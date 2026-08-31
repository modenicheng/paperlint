use paperlint::{config::Level, latex::span::Span, lint::diagnostic::Diagnostic, rule_id::RuleId};
use std::path::PathBuf;

#[test]
fn stable_rule_ids_when_iterated_then_match_public_api() {
    let ids: Vec<String> = RuleId::ALL.iter().map(ToString::to_string).collect();

    assert_eq!(
        ids,
        [
            "ACR001", "ACR002", "TERM001", "STYLE001", "FUNC001", "FUNC002", "STYLE002", "SYN001",
            "SYN002", "SYN003", "SYN004", "SYN005"
        ]
    );
}

#[test]
fn diagnostic_when_serialized_then_rule_id_is_public_string() {
    let diagnostic = Diagnostic {
        rule: RuleId::Acr001,
        severity: Level::Error,
        message: "acronym `RAG` used before definition".to_string(),
        span: Span {
            file: PathBuf::from("sections/introduction.tex"),
            start: 42,
            end: 45,
            line: 37,
            column: 12,
        },
    };

    let json = serde_json::to_value(diagnostic).expect("diagnostic serializes");

    assert_eq!(json["rule"], "ACR001");
    assert_eq!(json["severity"], "error");
}
