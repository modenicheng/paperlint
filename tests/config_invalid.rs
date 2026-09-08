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

#[test]
fn given_unsupported_nlp_tokenizer_when_parsing_then_it_fails_with_context() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[nlp]
tokenizer = "none"
pos = "jieba"
syntax = "none"
"#,
    );
    let error = result
        .expect_err("unsupported tokenizer must fail")
        .to_string();
    assert!(error.contains("tokenizer"), "{error}");
    assert!(error.contains("none"), "{error}");
}

#[test]
fn given_unsupported_nlp_pos_when_parsing_then_it_fails_with_context() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[nlp]
tokenizer = "jieba"
pos = "none"
syntax = "none"
"#,
    );
    let error = result
        .expect_err("unsupported POS backend must fail")
        .to_string();
    assert!(error.contains("pos"), "{error}");
    assert!(error.contains("none"), "{error}");
}

#[test]
fn given_unsupported_nlp_syntax_when_parsing_then_it_fails_with_context() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "jieba"
"#,
    );
    let error = result
        .expect_err("unsupported syntax backend must fail")
        .to_string();
    assert!(error.contains("syntax"), "{error}");
    assert!(error.contains("jieba"), "{error}");
}

#[test]
fn given_empty_lexicon_canonical_when_parsing_then_it_fails() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[[lexicon.entries]]
canonical = ""
kind = "term"
"#,
    );
    assert!(result.is_err());
}

#[test]
fn given_whitespace_lexicon_canonical_when_parsing_then_it_fails() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[[lexicon.entries]]
canonical = "   "
kind = "term"
"#,
    );
    assert!(result.is_err());
}

#[test]
fn given_empty_lexicon_alias_when_parsing_then_it_fails() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[[lexicon.entries]]
canonical = "dataset"
aliases = ["data set", ""]
kind = "term"
"#,
    );
    assert!(result.is_err());
}

#[test]
fn given_whitespace_lexicon_alias_when_parsing_then_it_fails() {
    let result = toml::from_str::<paperlint::config::RawPaperlintConfig>(
        r#"
[[lexicon.entries]]
canonical = "dataset"
aliases = ["  "]
kind = "term"
"#,
    );
    assert!(result.is_err());
}
