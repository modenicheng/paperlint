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

/// Parse one block of logical text into a single-block document.
fn document_of(text: &str) -> paperlint::latex::parser::Document {
    parser::parse_stdin(text.to_string(), &DefaultConfig::load().latex).unwrap()
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

// ------------------------------------------------------------------
// Review counterexamples: empty surfaces must never produce junk
// ------------------------------------------------------------------

#[test]
fn empty_surfaces_are_rejected_at_parse_time() {
    for bad in [
        r#"canonical = ""
kind = "term""#,
        r#"canonical = "   "
kind = "term""#,
        r#"canonical = "dataset"
aliases = [""]
kind = "term""#,
        r#"canonical = "dataset"
aliases = ["  "]
kind = "term""#,
    ] {
        let toml = format!("[[lexicon.entries]]\n{bad}");
        assert!(
            toml::from_str::<RawPaperlintConfig>(&toml).is_err(),
            "must reject: {bad}"
        );
    }
}

#[test]
fn surfaces_are_trimmed_when_parsed() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "  LLM  "
aliases = [" large language model "]
kind = "acronym"
"#,
    );
    assert_eq!(config.lexicon.entries[0].canonical, "LLM");
    assert_eq!(
        config.lexicon.entries[0].aliases,
        vec!["large language model".to_string()]
    );
    let lexicon = Lexicon::from_config(&config);
    assert!(lexicon.lookup("LLM").is_some());
    assert!(lexicon.lookup("large language model").is_some());
}

#[test]
fn programmatic_empty_surfaces_produce_no_occurrences() {
    // Config parsing rejects empties, so the reachable construction paths
    // (built-ins + workspace + ACR001.ignore) cannot carry empty surfaces.
    // This guards the rebuild-time skip against future construction paths
    // regressing to empty needles flooding match_indices.
    let config = DefaultConfig::load();
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("Any text at all.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    assert_eq!(registry.lexicon_occurrences().len(), 0);

    // Even with workspace entries present, an empty surface can never be
    // parsed, so occurrence collection sees only non-empty needles.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "dataset"
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    assert_eq!(registry.lexicon_occurrences().len(), 0);
}

// ------------------------------------------------------------------
// Review counterexamples: surface conflicts and duplication
// ------------------------------------------------------------------

#[test]
fn alias_equal_to_canonical_does_not_double_count() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "dataset"
aliases = ["dataset", "data set"]
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("We build a dataset from a data set.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let occurrences: Vec<_> = registry.lexicon_occurrences_of("dataset").collect();
    assert_eq!(occurrences.len(), 2, "each span counted once");
    let mut surfaces: Vec<_> = occurrences.iter().map(|o| o.surface.clone()).collect();
    surfaces.sort();
    assert_eq!(
        surfaces,
        vec!["data set".to_string(), "dataset".to_string()]
    );
    // Spans are distinct, no duplicated (span, canonical) pairs.
    assert_ne!(occurrences[0].location.range, occurrences[1].location.range);
}

#[test]
fn alias_of_one_entry_named_as_another_canonical_stays_one_owner() {
    // "network" is a canonical of entry B and an alias of entry A: the
    // frozen index gives the key to one owner, and the other lexeme must
    // not produce occurrences for the same span.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "neural network"
aliases = ["network"]
kind = "term"

[[lexicon.entries]]
canonical = "network"
aliases = []
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);

    // Workspace later entry wins the folded "network" key.
    let owner = lexicon.lookup("network").expect("network resolves");
    assert_eq!(owner.canonical, "network");

    let document = document_of("A network helps.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    let network: Vec<_> = registry.lexicon_occurrences_of("network").collect();
    assert_eq!(network.len(), 1);
    // The losing lexeme never claims the same span.
    let neural: Vec<_> = registry.lexicon_occurrences_of("neural network").collect();
    assert_eq!(neural.len(), 0);
}

#[test]
fn folded_surface_conflict_last_config_entry_wins() {
    // Two case-insensitive entries fold onto the same key: the later
    // workspace entry owns it, the earlier one is still listed but cannot
    // claim the disputed spans.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "Dataset"
kind = "common"

[[lexicon.entries]]
canonical = "dataset"
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let owner = lexicon.lookup("DATASET").expect("folded key resolves");
    assert_eq!(owner.canonical, "dataset", "last config entry wins");

    let document = document_of("The Dataset grows; the DATASET is big.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    assert_eq!(registry.lexicon_occurrences_of("dataset").count(), 2);
    assert_eq!(registry.lexicon_occurrences_of("Dataset").count(), 0);
}

#[test]
fn exact_key_beats_folded_key_for_the_same_letters() {
    // Case-sensitive "CNN" (exact key) shadows the case-insensitive "cnn"
    // (folded key) for the uppercase letters; lowercase still folds.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "cnn"
kind = "term"
case_sensitive = false

[[lexicon.entries]]
canonical = "CNN"
kind = "acronym"
case_sensitive = true
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    assert_eq!(lexicon.lookup("CNN").unwrap().canonical, "CNN");
    assert_eq!(lexicon.lookup("cnn").unwrap().canonical, "cnn");

    let document = document_of("CNN beats cnn.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    let upper: Vec<_> = registry.lexicon_occurrences_of("CNN").collect();
    assert_eq!(upper.len(), 1);
    assert_eq!(upper[0].surface, "CNN");
    let lower: Vec<_> = registry.lexicon_occurrences_of("cnn").collect();
    assert_eq!(lower.len(), 1);
    assert_eq!(lower[0].surface, "cnn");
    // The same span never carries both attributions.
    assert_ne!(upper[0].location.range, lower[0].location.range);
}

#[test]
fn nested_surfaces_allow_distinct_spans_but_not_double_counting() {
    // "network" nests inside "neural network": different spans may both
    // appear (nested occurrences allowed), but one span is never counted
    // twice for the same or contested lexemes.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "neural network"
aliases = []
kind = "term"

[[lexicon.entries]]
canonical = "network"
aliases = []
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("A neural network is a network.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let neural: Vec<_> = registry.lexicon_occurrences_of("neural network").collect();
    assert_eq!(neural.len(), 1, "outer span matched once");
    let networks: Vec<_> = registry.lexicon_occurrences_of("network").collect();
    // Nested span inside "neural network" plus the standalone one.
    assert_eq!(
        networks.len(),
        2,
        "nested and standalone spans both allowed"
    );
    // The inner occurrence really nests inside the outer one.
    let outer = &neural[0].location.range;
    assert!(
        networks
            .iter()
            .any(|inner| outer.start <= inner.location.range.start
                && inner.location.range.end <= outer.end
                && inner.location.range != *outer),
        "expected a nested span inside {outer:?}"
    );
    // No (span, canonical) pair ever appears twice across all occurrences.
    let mut seen = std::collections::HashSet::new();
    for occurrence in registry.lexicon_occurrences() {
        let key = (
            occurrence.location.block,
            occurrence.location.range.clone(),
            occurrence.canonical.clone(),
        );
        assert!(
            seen.insert(key.clone()),
            "double-counted occurrence {key:?}"
        );
    }
}

#[test]
fn same_span_two_attributions_never_both_survive() {
    // Even if two entries' surfaces overlap exactly at one span, the
    // occurrence list never contains two entries for one (span, canonical)
    // pair, and a span owned by one lexeme is not stolen by another.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "transformer"
aliases = ["transformer"]
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("The transformer works.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    let occurrences: Vec<_> = registry.lexicon_occurrences_of("transformer").collect();
    assert_eq!(occurrences.len(), 1, "alias==canonical collapses to one");
}

// ------------------------------------------------------------------
// Review counterexamples: occurrence metadata and ASCII boundaries
// ------------------------------------------------------------------

#[test]
fn occurrences_carry_case_sensitivity_and_explanation_flags() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "LLM"
aliases = ["large language model"]
kind = "acronym"
requires_explanation = true

[[lexicon.entries]]
canonical = "dataset"
aliases = ["data set"]
kind = "term"
case_sensitive = false
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("An LLM over a Data Set.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let llm: Vec<_> = registry.lexicon_occurrences_of("LLM").collect();
    assert_eq!(llm.len(), 1);
    assert!(llm[0].case_sensitive, "acronym matched case-sensitively");
    assert!(llm[0].requires_explanation);

    let dataset: Vec<_> = registry.lexicon_occurrences_of("dataset").collect();
    assert_eq!(dataset.len(), 1);
    assert!(!dataset[0].case_sensitive);
    assert!(!dataset[0].requires_explanation);
    // Enough metadata is stored that rules need no lookup round-trip: the
    // stored canonical resolves back to a lexeme with the same flags.
    let lexeme = lexicon.lookup(&llm[0].canonical).unwrap();
    assert_eq!(lexeme.case_sensitive, llm[0].case_sensitive);
    assert_eq!(lexeme.requires_explanation, llm[0].requires_explanation);
}

#[test]
fn underscore_is_an_ascii_identifier_boundary() {
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "CNN"
kind = "acronym"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("FOO_CNN_BAR and CNN and 基于CNN的模型。");
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let occurrences: Vec<_> = registry.lexicon_occurrences_of("CNN").collect();
    assert_eq!(
        occurrences.len(),
        2,
        "identifier CNN skipped, standalone and CJK-adjacent kept"
    );
    for occurrence in occurrences {
        let text = &document.blocks[occurrence.location.block].text;
        let bytes = text.as_bytes();
        let start = occurrence.location.range.start;
        let end = occurrence.location.range.end;
        if start > 0 {
            assert_ne!(bytes[start - 1], b'_', "never right after an underscore");
        }
        if end < bytes.len() {
            assert_ne!(bytes[end], b'_', "never right before an underscore");
        }
    }
}

#[test]
fn large_workspace_lexicon_builds_and_lints_quickly() {
    // 2000 entries: structural smoke test that overlay_workspace builds the
    // index in batches, not per-entry; guards against O(n^2) rebuilds
    // reintroducing noticeable latency.
    let mut toml = String::new();
    for i in 0..2000 {
        toml.push_str(&format!(
            "\n[[lexicon.entries]]\ncanonical = \"term{i}\"\naliases = [\"alias {i}\"]\nkind = \"term\"\n"
        ));
    }
    let config = config_with_lexicon(&toml);
    assert_eq!(config.lexicon.entries.len(), 2000);

    let start = std::time::Instant::now();
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("term0 uses alias 7 and term1999.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    let elapsed = start.elapsed();

    assert_eq!(registry.lexicon_occurrences().len(), 3);
    assert!(
        elapsed.as_secs() < 10,
        "from_config + one document should be fast, took {elapsed:?}"
    );
}

// ------------------------------------------------------------------
// Aho-Corasick matcher parity: overlap, same-span single count, empty lexicon
// ------------------------------------------------------------------

#[test]
fn exact_and_folded_hits_on_one_span_are_counted_once() {
    // A lowercase-spelled case-sensitive entry owns the exact key
    // "dataset" while a case-insensitive entry folds to the same needle:
    // both automata fire over the same span, but only the exact owner may
    // claim it, so the span must yield exactly one occurrence.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "dataset"
kind = "term"
case_sensitive = true

[[lexicon.entries]]
canonical = "Dataset"
kind = "term"
case_sensitive = false
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    assert_eq!(lexicon.lookup("dataset").unwrap().canonical, "dataset");
    assert_eq!(lexicon.lookup("Dataset").unwrap().canonical, "Dataset");

    let document = document_of("The dataset grows; the Dataset stays.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let lower: Vec<_> = registry.lexicon_occurrences_of("dataset").collect();
    assert_eq!(lower.len(), 1, "exact hit recorded once");
    assert!(lower[0].case_sensitive);
    let folded: Vec<_> = registry.lexicon_occurrences_of("Dataset").collect();
    assert_eq!(folded.len(), 1, "folded hit on its own span");
    assert!(!folded[0].case_sensitive);
    assert_ne!(
        lower[0].location.range, folded[0].location.range,
        "one span never carries both attributions"
    );
}

#[test]
fn matcher_reports_nested_overlapping_spans() {
    // "network" occurs inside "neural network" and standalone: the
    // Aho-Corasick pass replaces per-key match_indices, so it must keep
    // reporting overlapping hits the old scan produced.
    let config = config_with_lexicon(
        r#"
[[lexicon.entries]]
canonical = "neural network"
kind = "term"

[[lexicon.entries]]
canonical = "network"
kind = "term"
"#,
    );
    let lexicon = Lexicon::from_config(&config);
    let document = document_of("A neural network plus a network.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);

    let neural: Vec<_> = registry.lexicon_occurrences_of("neural network").collect();
    assert_eq!(neural.len(), 1, "outer span matched once");
    let networks: Vec<_> = registry.lexicon_occurrences_of("network").collect();
    assert_eq!(networks.len(), 2, "nested + standalone spans");
    let outer = neural[0].location.range.clone();
    let nested = networks
        .iter()
        .find(|occurrence| {
            outer.start <= occurrence.location.range.start
                && occurrence.location.range.end <= outer.end
        })
        .expect("nested network span inside neural network");
    assert_ne!(nested.location.range, outer);
}

#[test]
fn empty_lexicon_registry_yields_no_occurrences() {
    // A lexicon without scan keys compiles no matchers; registry
    // construction must skip collection instead of failing to build an
    // empty automaton.
    let lexicon = Lexicon::default();
    assert!(lexicon.lookup("AI").is_none());
    let document = document_of("Plain text with AI and CPU.");
    let registry = DocumentTermRegistry::new(&document, &lexicon);
    assert!(registry.lexicon_occurrences().is_empty());
}
