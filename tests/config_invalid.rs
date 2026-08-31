#[test]
fn given_unknown_option_when_parsing_then_it_fails() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[rules.STYLE001]
max_word = 45
"#,
    );
    assert!(result.is_err());
}
