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
