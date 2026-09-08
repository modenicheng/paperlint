use paperlint::config::{
    DefaultConfig, NlpPosBackend, NlpSyntaxBackend, NlpTokenizerBackend, PaperlintConfig,
    RawPaperlintConfig,
};

#[test]
fn given_no_config_when_loading_defaults_then_expected_rules_are_enabled() {
    let config = DefaultConfig::load();
    assert!(config.rules.acr001.level.is_enabled());
    assert!(config.rules.acr002.level.is_enabled());
    assert!(config.rules.term001.level.is_enabled());
    assert!(config.rules.style001.level.is_enabled());
    assert_eq!(config.nlp.tokenizer, NlpTokenizerBackend::Jieba);
    assert_eq!(config.nlp.pos, NlpPosBackend::Jieba);
    assert_eq!(config.nlp.syntax, NlpSyntaxBackend::None);
}

#[test]
fn given_bool_shorthand_when_merging_then_rule_is_disabled() {
    let defaults = DefaultConfig::load();
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules]
acr002 = false
"#,
    )
    .expect("raw config");
    let config = PaperlintConfig::from_raw(raw, defaults);
    assert!(!config.rules.acr002.level.is_enabled());
}

#[test]
fn acronym_rule_sections_and_documented_keys_are_accepted() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.ACR001]
level = "error"
min_length = 3
ignore = ["API"]

[rules.ACR002]
level = "warning"
min_usages_after_definition = 2
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, DefaultConfig::load());

    assert_eq!(config.rules.acr001.min_length, 3);
    assert_eq!(config.rules.acr001.ignore, ["API"]);
    assert_eq!(config.rules.acr002.min_usages_after_definition, 2);
}

#[test]
fn acr002_old_min_occurrences_alias_is_accepted() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.acr002]
level = "warning"
min_occurrences = 3
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, DefaultConfig::load());

    assert_eq!(config.rules.acr002.min_usages_after_definition, 3);
}

#[test]
fn style001_old_max_words_alias_keeps_default_chinese_limit() {
    let raw: RawPaperlintConfig = toml::from_str(
        r#"
[rules.STYLE001]
level = "warning"
max_words = 7
"#,
    )
    .unwrap();
    let config = PaperlintConfig::from_raw(raw, DefaultConfig::load());
    assert_eq!(config.rules.style001.max_chars, 80);
    assert_eq!(config.rules.style001.max_english_words, 7);
}

#[test]
fn given_nlp_config_when_serializing_then_backends_are_structured_values() {
    let config = DefaultConfig::load();

    let value = serde_json::to_value(&config.nlp).expect("NLP config serializes");

    assert_eq!(value["tokenizer"], "jieba");
    assert_eq!(value["pos"], "jieba");
    assert_eq!(value["syntax"], "none");
}

#[test]
fn given_documented_full_example_when_parsing_then_it_is_valid_config() {
    let rules_doc = include_str!("../docs/rules.md");
    let full_example = extract_full_example_toml(rules_doc);

    let raw: RawPaperlintConfig =
        toml::from_str(full_example).expect("documented full paperlint.toml parses");
    let config = PaperlintConfig::from_raw(raw, DefaultConfig::load());

    assert_eq!(config.rules.func001.min_sentence_tokens, 10);
    assert!(!config.rules.syn005.require_dependency);
}

fn extract_full_example_toml(document: &str) -> &str {
    let heading = document
        .find("## Full example")
        .expect("full example heading");
    let after_heading = &document[heading..];
    let fence_start = after_heading
        .find("```toml\n")
        .expect("full example TOML fence")
        + "```toml\n".len();
    let after_fence = &after_heading[fence_start..];
    let fence_end = after_fence.find("\n```").expect("full example fence end");
    &after_fence[..fence_end]
}
