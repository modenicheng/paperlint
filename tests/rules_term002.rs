use paperlint::{
    config::{DefaultConfig, Level, PaperlintConfig, RawPaperlintConfig},
    latex::parser::{self, Document},
    lint::{diagnostic::Diagnostic, engine::RuleEngine, registry::RuleRegistry},
    rule_id::RuleId,
    text::lexicon::LexemeKind,
};
use std::{fs, path::Path};
use tempfile::tempdir;

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn base_config() -> PaperlintConfig {
    let mut config = DefaultConfig::load();
    config.rules.acr001.level = Level::Off;
    config.rules.acr002.level = Level::Off;
    config.rules.term001.level = Level::Off;
    config.rules.style001.level = Level::Off;
    config.rules.punc002.level = Level::Off;
    config
}

fn entry(
    canonical: &str,
    kind: LexemeKind,
    requires_explanation: bool,
) -> paperlint::config::LexiconEntryConfig {
    paperlint::config::LexiconEntryConfig {
        canonical: canonical.to_string(),
        aliases: Vec::new(),
        kind,
        case_sensitive: None,
        requires_explanation,
    }
}

fn term(canonical: &str) -> paperlint::config::LexiconEntryConfig {
    entry(canonical, LexemeKind::Term, true)
}

fn lint(source: &str, configure: impl FnOnce(&mut PaperlintConfig)) -> (Document, Vec<Diagnostic>) {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, source);
    let mut config = base_config();
    configure(&mut config);
    let document = parser::parse(main.clone(), &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    (document, diagnostics)
}

fn span_text<'a>(document: &'a Document, diagnostic: &Diagnostic) -> &'a str {
    let source = document.source(&diagnostic.span.file).unwrap();
    &source.text[diagnostic.span.start..diagnostic.span.end]
}

#[test]
fn chinese_trigger_forms_count_as_explained() {
    for (name, source) in [
        ("shi", "本文研究的智能体是一种自主实体。"),
        ("shizhi", "本文研究智能体，指的是自主实体。"),
        ("zhi", "本文研究智能体，指自主实体。"),
        ("ji", "本文研究智能体，即自主实体。"),
        ("dingyi", "本文研究智能体，定义为自主实体。"),
        ("biaoshi", "本文研究智能体，表示自主实体。"),
        ("suowei", "所谓智能体，指自主实体。"),
        ("chengwei", "本文称智能体为自主实体。"),
    ] {
        let (_, diagnostics) = lint(source, |config| {
            config.lexicon.entries.push(term("智能体"));
        });
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
    }
}

#[test]
fn english_trigger_forms_count_as_explained() {
    for (name, source) in [
        ("refers", "The attention mechanism refers to the weights."),
        ("defined", "The gating unit is defined as a sigmoid layer."),
        ("denotes", "This symbol denotes the gradient norm."),
        ("stands", "The tag stands for the loss term."),
        (
            "also_known",
            "The critic (also known as the value net) scores states.",
        ),
        ("ie", "The critic, i.e., the value net, scores states."),
        ("colon", "The critic: a value network for scoring states."),
    ] {
        let (_, diagnostics) = lint(source, |config| {
            config.lexicon.entries.push(term("critic"));
        });
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
    }
}

#[test]
fn hard_negatives_stay_unexplained() {
    for (name, source) in [
        ("possessive-subject", "该智能体的性能是关键指标。"),
        ("other-subject-represents-x", "该方法表示智能体的分布。"),
        ("no-trigger", "本文研究智能体完成复杂任务。"),
    ] {
        let (document, diagnostics) = lint(source, |config| {
            config.lexicon.entries.push(term("智能体"));
        });
        assert_eq!(diagnostics.len(), 1, "{name}: {diagnostics:?}");
        assert_eq!(diagnostics[0].rule, RuleId::Term002);
        assert_eq!(diagnostics[0].severity, Level::Warning);
        assert_eq!(
            diagnostics[0].message,
            "technical term `智能体` first used without a nearby explanation"
        );
        assert_eq!(span_text(&document, &diagnostics[0]), "智能体", "{name}");
    }
}

#[test]
fn structured_definitions_count_as_explained() {
    for (name, source, canonical) in [
        ("acronym-span", "大语言模型（LLM）推动了研究。", "LLM"),
        (
            "chinese-full-form",
            "大语言模型（Large Language Model，LLM）推动了研究。",
            "大语言模型",
        ),
        (
            "english-full-form",
            "Large Language Model (LLM) changed the field.",
            "large language model",
        ),
    ] {
        let (_, diagnostics) = lint(source, |config| {
            config.lexicon.entries.push(term(canonical));
        });
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
    }
}

#[test]
fn later_explanation_never_cancels_the_first_use_gap() {
    let (document, diagnostics) = lint(
        "本文研究智能体。后文说明：智能体是指自主实体。",
        |config| {
            config.lexicon.entries.push(term("智能体"));
        },
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(span_text(&document, &diagnostics[0]), "智能体");
}

#[test]
fn one_diagnostic_per_term_with_aliases_and_case_folding() {
    let (document, diagnostics) = lint("使用critic机制。Critic很重要。", |config| {
        let mut critic = term("critic");
        critic.aliases = vec!["value net".to_string()];
        config.lexicon.entries.push(critic);
    });
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(span_text(&document, &diagnostics[0]), "critic");
}

#[test]
fn common_kind_and_requires_false_are_never_flagged() {
    for (name, configured) in [
        ("common-kind", entry("智能体", LexemeKind::Common, true)),
        ("requires-false", entry("智能体", LexemeKind::Term, false)),
    ] {
        let (_, diagnostics) = lint("本文研究智能体完成复杂任务。", |config| {
            config.lexicon.entries.push(configured.clone());
        });
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
    }
}

#[test]
fn cross_include_reports_span_in_child_file() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    write(&main, "\\input{child}正文接续。");
    write(&child, "本文研究智能体。");
    let mut config = base_config();
    config.lexicon.entries.push(term("智能体"));
    let document = parser::parse(main.clone(), &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].span.file, child.canonicalize().unwrap());
    assert_eq!(span_text(&document, &diagnostics[0]), "智能体");
}

#[test]
fn small_context_chars_shrinks_the_trigger_window() {
    // The trigger 指的是 sits four characters after the term; a window of
    // one character cannot see it, so the first use is unexplained.
    let (_, diagnostics) = lint(
        "本文研究智能体，指的是自主实体。",
        |config| {
            config.lexicon.entries.push(term("智能体"));
            config.rules.term002.context_chars = 1;
        },
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
}

#[test]
fn off_level_suppresses_the_rule() {
    let (_, diagnostics) = lint("本文研究智能体。", |config| {
        config.lexicon.entries.push(term("智能体"));
        config.rules.term002.level = Level::Off;
    });
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn raw_config_sets_level_context_chars_and_lexicon_terms() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.TERM002]
level = "error"
context_chars = 50

[[lexicon.entries]]
canonical = "智能体"
kind = "term"
requires_explanation = true
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, base_config());
    assert_eq!(config.rules.term002.level, Level::Error);
    assert_eq!(config.rules.term002.context_chars, 50);
    assert_eq!(config.lexicon.entries.len(), 1);

    let (document, diagnostics) = lint("本文研究智能体。", |config| {
        config.lexicon.entries = vec![term("智能体")];
        config.rules.term002.level = Level::Error;
        config.rules.term002.context_chars = 50;
    });
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Level::Error);
    assert_eq!(span_text(&document, &diagnostics[0]), "智能体");
}

#[test]
fn default_context_chars_is_one_hundred() {
    let config = DefaultConfig::load();
    assert_eq!(config.rules.term002.level, Level::Warning);
    assert_eq!(config.rules.term002.context_chars, 100);
}

#[test]
fn cli_enable_disable_overrides_target_term002() {
    let enabled = base_config().with_overrides(&["TERM002".to_string()], &[]);
    assert_eq!(enabled.rules.term002.level, Level::Warning);
    let disabled = base_config().with_overrides(&[], &["TERM002".to_string()]);
    assert_eq!(disabled.rules.term002.level, Level::Off);
}

#[test]
fn stdin_input_reports_term002() {
    let mut config = base_config();
    config.lexicon.entries.push(term("智能体"));
    let document = parser::parse_stdin("本文研究智能体。".to_string(), &config.latex).unwrap();
    let diagnostics = RuleEngine::new(RuleRegistry::default()).run(&document, &config);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].rule, RuleId::Term002);
}
