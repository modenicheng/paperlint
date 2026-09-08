use super::{test_support::*, *};
use proptest::prelude::*;

const PHRASE: &str = "跨模态语义蒸馏网络";

#[test]
fn emits_repeated_phrase_with_exact_ranges_and_count() {
    let phrase_len = PHRASE.len();
    let tokens = phrase_tokens(0, 0)
        .into_iter()
        .chain(phrase_tokens(0, phrase_len));
    let sentence = sentence_at(
        located(0, 0..phrase_len * 2),
        &(PHRASE.repeat(2)),
        tokens.collect(),
    );

    let occurrences = collect(&analysis(vec![paragraph(0, vec![sentence])]));
    let found = matching(&occurrences, PHRASE);

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].location, located(0, 0..phrase_len));
    assert_eq!(found[1].location, located(0, phrase_len..phrase_len * 2));
    assert_repetition(&found, 2);
}

#[test]
fn counts_exact_surfaces_across_blocks_and_retains_every_location() {
    let phrase_len = PHRASE.len();
    let paragraphs = vec![
        paragraph(
            0,
            vec![sentence_at(
                located(0, 0..phrase_len),
                PHRASE,
                phrase_tokens(0, 0),
            )],
        ),
        paragraph(
            1,
            vec![sentence_at(
                located(1, 0..phrase_len),
                PHRASE,
                phrase_tokens(1, 0),
            )],
        ),
    ];

    let occurrences = collect(&analysis(paragraphs));
    let found = matching(&occurrences, PHRASE);

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].location, located(0, 0..phrase_len));
    assert_eq!(found[1].location, located(1, 0..phrase_len));
    assert_repetition(&found, 2);
}

#[test]
fn retains_locations_for_repeated_contained_phrases() {
    let phrase_len = PHRASE.len();
    let tokens = phrase_tokens(0, 0)
        .into_iter()
        .chain(phrase_tokens(0, phrase_len));
    let sentence = sentence_at(
        located(0, 0..phrase_len * 2),
        &PHRASE.repeat(2),
        tokens.collect(),
    );
    let occurrences = collect(&analysis(vec![paragraph(0, vec![sentence])]));

    let prefix = matching(&occurrences, "跨模态语义");
    let suffix = matching(&occurrences, "语义蒸馏网络");

    assert_eq!(prefix.len(), 2);
    assert_eq!(prefix[0].location.range, 0..15);
    assert_eq!(prefix[1].location.range, phrase_len..phrase_len + 15);
    assert_eq!(suffix.len(), 2);
    assert_repetition(&prefix, 2);
    assert_repetition(&suffix, 2);
}

#[test]
fn rejects_singleton_and_generic_sentence() {
    let singleton = sentence_at(located(0, 0..PHRASE.len()), PHRASE, phrase_tokens(0, 0));
    let generic_text = "本文提出一种方法。";
    let generic = sentence_at(
        located(0, PHRASE.len()..PHRASE.len() + generic_text.len()),
        generic_text,
        vec![
            token(
                "本文",
                PosTag::Pronoun,
                located(0, PHRASE.len()..PHRASE.len() + 6),
            ),
            token(
                "提出",
                PosTag::Verb,
                located(0, PHRASE.len() + 6..PHRASE.len() + 12),
            ),
            token(
                "一种",
                PosTag::Numeral,
                located(0, PHRASE.len() + 12..PHRASE.len() + 18),
            ),
            token(
                "方法",
                PosTag::Noun,
                located(0, PHRASE.len() + 18..PHRASE.len() + 24),
            ),
            token(
                "。",
                PosTag::Punctuation,
                located(0, PHRASE.len() + 24..PHRASE.len() + 27),
            ),
        ],
    );

    let found = collect(&analysis(vec![paragraph(0, vec![singleton, generic])]));

    assert!(found.is_empty());
}

#[test]
fn rejects_whitespace_and_punctuation_crossings() {
    let spaced = sentence_at(
        located(0, 0.."跨模态 语义跨模态 语义".len()),
        "跨模态 语义跨模态 语义",
        vec![
            token("跨模态", PosTag::Noun, located(0, 0..9)),
            token("语义", PosTag::Noun, located(0, 10..16)),
            token("跨模态", PosTag::Noun, located(0, 16..25)),
            token("语义", PosTag::Noun, located(0, 26..32)),
        ],
    );
    let punctuated_text = "跨模态，语义跨模态，语义";
    let punctuated = sentence_at(
        located(1, 0..punctuated_text.len()),
        punctuated_text,
        vec![
            token("跨模态", PosTag::Noun, located(1, 0..9)),
            token("，", PosTag::Punctuation, located(1, 9..12)),
            token("语义", PosTag::Noun, located(1, 12..18)),
            token("跨模态", PosTag::Noun, located(1, 18..27)),
            token("，", PosTag::Punctuation, located(1, 27..30)),
            token("语义", PosTag::Noun, located(1, 30..36)),
        ],
    );

    let found = collect(&analysis(vec![
        paragraph(0, vec![spaced]),
        paragraph(1, vec![punctuated]),
    ]));

    assert!(matching(&found, "跨模态语义").is_empty());
}

#[test]
fn rejects_sentence_and_block_crossing_windows() {
    let sentence_parts = vec![
        one_token_sentence(0, 0, "跨模态"),
        one_token_sentence(0, 9, "语义"),
        one_token_sentence(0, 15, "跨模态"),
        one_token_sentence(0, 24, "语义"),
    ];
    let block_parts = vec![
        paragraph(1, vec![one_token_sentence(1, 0, "跨模态")]),
        paragraph(2, vec![one_token_sentence(2, 0, "语义")]),
        paragraph(3, vec![one_token_sentence(3, 0, "跨模态")]),
        paragraph(4, vec![one_token_sentence(4, 0, "语义")]),
    ];
    let mut paragraphs = vec![paragraph(0, sentence_parts)];
    paragraphs.extend(block_parts);

    let found = collect(&analysis(paragraphs));

    assert!(found.is_empty());
}

#[test]
fn rejects_adjacent_ranges_from_different_blocks() {
    let text = "跨模态语义跨模态语义";
    let tokens = vec![
        token("跨模态", PosTag::Noun, located(0, 0..9)),
        token("语义", PosTag::Noun, located(1, 9..15)),
        token("跨模态", PosTag::Noun, located(0, 15..24)),
        token("语义", PosTag::Noun, located(1, 24..30)),
    ];
    let sentence = sentence_at(located(0, 0..text.len()), text, tokens);

    let found = collect(&analysis(vec![paragraph(0, vec![sentence])]));

    assert!(found.is_empty());
}

#[test]
fn rejects_ascii_arbitrary_character_and_non_nominal_grams() {
    let ascii = repeated_pair("deep", "model", PosTag::Noun);
    let arbitrary = repeated_tokens(&["跨", "模", "态", "语"], PosTag::Noun);
    let verbal = repeated_pair("提出", "方法", PosTag::Verb);

    assert!(collect(&analysis(vec![paragraph(0, vec![ascii])])).is_empty());
    assert!(collect(&analysis(vec![paragraph(0, vec![arbitrary])])).is_empty());
    assert!(collect(&analysis(vec![paragraph(0, vec![verbal])])).is_empty());
}

#[test]
fn accepts_adjectives_and_enforces_scalar_bounds() {
    let adjective = repeated_pair("跨模态", "模型", PosTag::Adjective);
    let short = repeated_pair("研究", "生", PosTag::Noun);
    let shortest = repeated_pair("研究", "方法", PosTag::Noun);
    let longest = repeated_pair(&"术".repeat(23), "语", PosTag::Noun);
    let too_long = repeated_pair(&"术".repeat(24), "语", PosTag::Noun);

    assert_eq!(matching(&collect_one(adjective), "跨模态模型").len(), 2);
    assert!(collect_one(short).is_empty());
    assert_eq!(matching(&collect_one(shortest), "研究方法").len(), 2);
    assert_eq!(collect_one(longest).len(), 2);
    assert!(collect_one(too_long).is_empty());
}

#[test]
fn groups_by_byte_exact_surface_without_normalization() {
    let composed = pair_sentence("研究", "é模型", 0);
    let decomposed = pair_sentence("研究", "e\u{301}模型", composed.location.range.end);

    let found = collect(&analysis(vec![paragraph(0, vec![composed, decomposed])]));

    assert!(found.is_empty());
}

#[test]
fn output_order_is_deterministic() {
    let sentence = repeated_tokens(&["智能", "分析", "系统"], PosTag::Noun);
    let analysis = analysis(vec![paragraph(0, vec![sentence])]);

    let first = collect(&analysis);
    let second = collect(&analysis);

    assert_eq!(first, second);
    assert!(
        first
            .windows(2)
            .all(|pair| { pair[0].location.position() <= pair[1].location.position() })
    );
}

#[test]
fn enumerates_at_most_five_windows_per_token_start() {
    let token_count = 10_000;

    let enumerated = candidate_ranges(token_count).count();

    assert_eq!(enumerated, token_count * 5 - 15);
    assert!(enumerated <= token_count * 5);
}

proptest! {
    #[test]
    fn generated_window_ranges_are_always_bounded(token_count in 0usize..5_000) {
        let mut enumerated = 0;
        for (start, end) in candidate_ranges(token_count) {
            prop_assert!((MIN_WINDOW..=MAX_WINDOW).contains(&(end - start)));
            prop_assert!(end <= token_count);
            enumerated += 1;
        }
        prop_assert!(enumerated <= token_count * 5);
    }
}
