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
