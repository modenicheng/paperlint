use paperlint::lint::terms::{CandidateScore, TermCandidateEvidence, TermCandidateKindHint};
use proptest::{prelude::*, test_runner::Config as ProptestConfig};

#[test]
fn scores_each_evidence_weight_when_single_signal() {
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::AllCaps]).value(),
        40
    );
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::CamelOrPascalCase]).value(),
        35
    );
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::LetterDigit]).value(),
        35
    );
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::MixedScript]).value(),
        45
    );
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::DefinitionContext]).value(),
        35
    );
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::Emphasis]).value(),
        25
    );
    assert_eq!(
        CandidateScore::from_evidence(&[TermCandidateEvidence::RepeatedNgram { occurrences: 2 }])
            .value(),
        30
    );
}

#[test]
fn score_edges_cover_empty_duplicate_and_large_repetition_inputs() {
    assert_eq!(CandidateScore::from_evidence(&[]).value(), 0);
    let repeated_evidence = vec![
        TermCandidateEvidence::LetterDigit,
        TermCandidateEvidence::LetterDigit,
        TermCandidateEvidence::RepeatedNgram { occurrences: 2 },
        TermCandidateEvidence::RepeatedNgram { occurrences: 7 },
    ];
    assert_eq!(
        CandidateScore::from_evidence(&repeated_evidence).value(),
        75
    );
    let saturated_evidence = vec![
        TermCandidateEvidence::AllCaps,
        TermCandidateEvidence::MixedScript,
        TermCandidateEvidence::DefinitionContext,
        TermCandidateEvidence::Emphasis,
        TermCandidateEvidence::RepeatedNgram {
            occurrences: usize::MAX,
        },
    ];
    assert_eq!(
        CandidateScore::from_evidence(&saturated_evidence).value(),
        100
    );
}

#[test]
fn applies_kind_precedence_and_uses_unknown_only_for_empty_evidence() {
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[TermCandidateEvidence::DefinitionContext]),
        TermCandidateKindHint::DefinitionLike
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[
            TermCandidateEvidence::MixedScript,
            TermCandidateEvidence::DefinitionContext,
        ]),
        TermCandidateKindHint::DefinitionLike
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[
            TermCandidateEvidence::RepeatedNgram { occurrences: 2 },
            TermCandidateEvidence::MixedScript,
        ]),
        TermCandidateKindHint::MixedScript
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[
            TermCandidateEvidence::AllCaps,
            TermCandidateEvidence::RepeatedNgram { occurrences: 2 },
        ]),
        TermCandidateKindHint::RepeatedNgram
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[TermCandidateEvidence::AllCaps]),
        TermCandidateKindHint::EnglishAcronym
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[TermCandidateEvidence::CamelOrPascalCase]),
        TermCandidateKindHint::EnglishIdentifier
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[TermCandidateEvidence::LetterDigit]),
        TermCandidateKindHint::EnglishIdentifier
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[TermCandidateEvidence::Emphasis]),
        TermCandidateKindHint::ChineseCompound
    );
    assert_eq!(
        TermCandidateKindHint::from_evidence(&[]),
        TermCandidateKindHint::Unknown
    );
}

prop_compose! {
    fn evidence_strategy()
        (choice in prop_oneof![
            Just(TermCandidateEvidence::AllCaps),
            Just(TermCandidateEvidence::CamelOrPascalCase),
            Just(TermCandidateEvidence::LetterDigit),
            Just(TermCandidateEvidence::MixedScript),
            Just(TermCandidateEvidence::DefinitionContext),
            Just(TermCandidateEvidence::Emphasis),
            (0_usize..1_000).prop_map(|occurrences| {
                TermCandidateEvidence::RepeatedNgram { occurrences }
            }),
        ]) -> TermCandidateEvidence {
            choice
        }
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn score_never_exceeds_one_hundred(evidence in prop::collection::vec(evidence_strategy(), 0..40)) {
        let score = CandidateScore::from_evidence(&evidence);
        prop_assert!(score.value() <= 100);
    }

    #[test]
    fn arbitrary_permutation_does_not_change_score_or_kind(
        items in prop::collection::vec((0_u64..1_000, evidence_strategy()), 1..40)
    ) {
        let evidence: Vec<_> = items.iter().map(|(_, item)| item.clone()).collect();
        let mut keyed_evidence = items;
        keyed_evidence.sort_by_key(|(priority, item)| (*priority, item.clone()));
        let permuted_evidence: Vec<_> = keyed_evidence
            .into_iter()
            .map(|(_, item)| item)
            .collect();

        prop_assert_eq!(
            CandidateScore::from_evidence(&evidence),
            CandidateScore::from_evidence(&permuted_evidence)
        );
        prop_assert_eq!(
            TermCandidateKindHint::from_evidence(&evidence),
            TermCandidateKindHint::from_evidence(&permuted_evidence)
        );
    }
}
