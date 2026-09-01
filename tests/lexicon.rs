use paperlint::{
    config::{DefaultConfig, LexiconConfig, PaperlintConfig, RawPaperlintConfig},
    latex::parser,
    lint::context::{DocumentTermRegistry, LintContext, ReadingPosition},
    text::lexicon::{LexemeKind, LexemeSource, Lexicon},
};

fn config_with_lexicon(toml: &str) -> PaperlintConfig {
    let raw: RawPaperlintConfig = toml::from_str(toml).expect("raw config");
    PaperlintConfig::from_raw(raw, DefaultConfig::load())
}

#[test]
fn default_config_has_no_workspace_lexicon_entries() {
    let config = DefaultConfig::load();
    assert_eq!(
        config.lexicon,
        LexiconConfig {
            entries: Vec::new()
        }
    );
}

#[test]
fn workspace_lexicon_entries_are_parsed_and_merged() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "LLM"
aliases = ["large language model"]
kind = "acronym"

[[lexicon.entries]]
canonical = "dataset"
aliases = ["data set"]
kind = "term"
requires_explanation = true
"#,
    );
    assert_eq!(config.lexicon.entries.len(), 2);
    assert_eq!(config.lexicon.entries[0].canonical, "LLM");
    assert_eq!(
        config.lexicon.entries[0].aliases,
        vec!["large language model".to_string()]
    );
    assert_eq!(config.lexicon.entries[0].kind, LexemeKind::Acronym);
    assert!(config.lexicon.entries[0].case_sensitive.is_none());
    assert!(!config.lexicon.entries[0].requires_explanation);

    assert_eq!(config.lexicon.entries[1].kind, LexemeKind::Term);
    assert!(config.lexicon.entries[1].requires_explanation);

    let lexicon = Lexicon::from_config(&config);
    let llm = lexicon.lookup("LLM").expect("LLM resolves");
    assert_eq!(llm.source, LexemeSource::Workspace);
    assert!(llm.case_sensitive, "acronyms default to case-sensitive");
    assert!(lexicon.lookup("large language model").is_some());
    // Case-insensitive term: alias and canonical fold.
    assert!(lexicon.lookup("Dataset").is_some());
    assert!(lexicon.lookup("DATA SET").is_some());
    // Built-ins survive next to workspace entries.
    assert!(lexicon.lookup("CPU").is_some());
}

#[test]
fn workspace_entry_overrides_builtin_of_same_canonical() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "AI"
kind = "term"
case_sensitive = false
requires_explanation = true
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let found = lexicon.lookup("ai").expect("case-insensitive override");
    assert_eq!(found.kind, LexemeKind::Term);
    assert_eq!(found.source, LexemeSource::Workspace);
    assert!(found.requires_explanation);
    assert_eq!(
        lexicon
            .entries()
            .filter(|lexeme| lexeme.canonical == "AI")
            .count(),
        1,
        "override replaces, not duplicates"
    );
}

#[test]
fn acr001_ignore_acronyms_stack_into_lexicon() {
    let config = config_with_lexicon(
        r#"
[rules.ACR001]
level = "error"
min_length = 2
ignore = ["XYZ", "AI"]
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let stacked = lexicon.lookup("XYZ").expect("stacked acronym");
    assert_eq!(stacked.kind, LexemeKind::Acronym);
    assert_eq!(stacked.source, LexemeSource::AcronymIgnore);
    assert_eq!(
        lexicon.lookup("AI").expect("builtin kept").source,
        LexemeSource::Builtin
    );
}

#[test]
fn lexicon_config_rejects_unknown_fields() {
    let result = toml::from_str::<RawPaperlintConfig>(
        r#"
[[lexicon.entries]]
canonical = "LLM"
kind = "acronym"
unknown_key = true
"#,
    );
    assert!(result.is_err());
}

#[test]
fn lexicon_entry_requires_canonical_and_kind() {
    assert!(
        toml::from_str::<RawPaperlintConfig>(
            r#"
[[lexicon.entries]]
kind = "acronym"
"#,
        )
        .is_err()
    );
    assert!(
        toml::from_str::<RawPaperlintConfig>(
            r#"
[[lexicon.entries]]
canonical = "LLM"
"#,
        )
        .is_err()
    );
    assert!(
        toml::from_str::<RawPaperlintConfig>(
            r#"
[[lexicon.entries]]
canonical = "LLM"
kind = "not-a-kind"
"#,
        )
        .is_err()
    );
}

#[test]
fn registry_reports_acronyms_and_lexicon_occurrences_in_reading_order() {
    let config = DefaultConfig::load();
    let lexicon = Lexicon::from_config(&config);
    let main = r"\section{Intro}
卷积神经网络（Convolutional Neural Network，CNN）是常见模型。CNN 效果好。
CNN 与 CPU 都出现了。
";
    let document = parser::parse_stdin(main.to_string(), &config.latex).unwrap();
    let context = LintContext::new(&document, &config, &lexicon);
    let registry = context.registry();

    // Acronym analysis: definition + usages across blocks in reading order.
    let first_cnn = registry.first_definition("CNN").expect("CNN defined");
    assert_eq!(first_cnn.acronym, "CNN");
    let after: Vec<_> = registry
        .usages_after("CNN", first_cnn.location.position())
        .collect();
    assert_eq!(after.len(), 2, "CNN used twice after definition");
    assert!(after.iter().all(|usage| !usage.is_definition));
    assert!(
        after[0].location.position() < after[1].location.position(),
        "usages ordered by reading position"
    );
    assert!(registry.first_definition("CPU").is_none());

    // Lexicon-resolved occurrences: builtin CPU must be found with location.
    let cpu_occurrences: Vec<_> = registry.lexicon_occurrences_of("CPU").collect();
    assert_eq!(cpu_occurrences.len(), 1);
    let first = registry.first_lexicon_occurrence("CPU").expect("first CPU");
    assert_eq!(first.surface, "CPU");
    assert_eq!(first.canonical, "CPU");
    assert_eq!(first.kind, LexemeKind::Acronym);
    assert_eq!(first.source, LexemeSource::Builtin);
    assert_eq!(first.location, cpu_occurrences[0].location);

    // Reading position ordering is total: block index then byte offset.
    let mut sorted: Vec<_> = registry.lexicon_occurrences().to_vec();
    sorted.sort_by_key(|occurrence| occurrence.location.position());
    assert_eq!(
        sorted.len(),
        registry.lexicon_occurrences().len(),
        "occurrences already in reading order"
    );

    // Location -> span mapping works for both query kinds.
    assert!(
        context
            .span(first.location.block, first.location.range.clone())
            .is_some()
    );
}

#[test]
fn lexicon_matching_respects_word_boundaries_for_ascii() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "AIM"
kind = "acronym"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let main = "AIM improves; but CLAIM must not match, nor does ai.";
    let document = parser::parse_stdin(main.to_string(), &config.latex).unwrap();
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let aim: Vec<_> = registry.lexicon_occurrences_of("AIM").collect();
    assert_eq!(aim.len(), 1, "only standalone AIM matches");
    assert_eq!(
        &document.blocks[aim[0].location.block].text[aim[0].location.range.clone()],
        "AIM"
    );

    // Builtin AI is case-sensitive, so lowercase "ai" does not match.
    assert_eq!(registry.lexicon_occurrences_of("AI").count(), 0);
}

#[test]
fn reading_position_orders_block_then_byte() {
    let early = ReadingPosition { block: 0, byte: 50 };
    let late_block = ReadingPosition { block: 1, byte: 0 };
    let late_byte = ReadingPosition { block: 0, byte: 60 };
    assert!(early < late_byte);
    assert!(early < late_block);
    assert!(late_byte < late_block);
}
