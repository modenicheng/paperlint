use paperlint::config::{DefaultConfig, PaperlintConfig, RawPaperlintConfig};

#[test]
fn given_no_config_when_loading_defaults_then_expected_rules_are_enabled() {
    let config = DefaultConfig::load();
    assert!(config.rules.acr001.level.is_enabled());
    assert!(config.rules.acr002.level.is_enabled());
    assert!(config.rules.term001.level.is_enabled());
    assert!(config.rules.style001.level.is_enabled());
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
