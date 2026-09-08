use super::{
    model::{RawTermOccurrence, TermCandidate, TermCandidateEvidence, TermCandidateKindHint},
    score::normalized_evidence,
};
use crate::latex::span::LocatedRange;
use proptest::prelude::*;

fn raw(surface: &str, evidence: Vec<TermCandidateEvidence>) -> RawTermOccurrence {
    RawTermOccurrence {
        surface: surface.to_string(),
        location: LocatedRange {
            block: 0,
            range: 4..13,
        },
        evidence,
    }
}

#[test]
fn raw_occurrence_builds_completed_candidate_with_read_only_accessors() {
    let result = TermCandidate::try_from(raw(
        "GraphRAG2",
        vec![
            TermCandidateEvidence::LetterDigit,
            TermCandidateEvidence::MixedScript,
        ],
    ));
    let Ok(candidate) = result else {
        panic!("non-empty evidence should emit candidate");
    };

    assert_eq!(
        candidate.location(),
        &LocatedRange {
            block: 0,
            range: 4..13
        }
    );
    assert_eq!(candidate.surface(), "GraphRAG2");
    assert_eq!(candidate.kind_hint(), TermCandidateKindHint::MixedScript);
    assert_eq!(candidate.score().value(), 80);
    assert_eq!(
        candidate.evidence(),
        &[
            TermCandidateEvidence::LetterDigit,
            TermCandidateEvidence::MixedScript,
        ]
    );
}

#[test]
fn rejects_empty_evidence_before_completed_candidate_construction() {
    assert!(TermCandidate::try_from(raw("GraphRAG2", Vec::new())).is_err());
    let evidence = normalized_evidence(Vec::new());
    assert!(evidence.is_empty());
    assert_eq!(
        TermCandidateKindHint::from_evidence(&evidence),
        TermCandidateKindHint::Unknown
    );
}

#[test]
fn deduplicates_evidence_and_merges_repeated_ngram_occurrences() {
    let result = TermCandidate::try_from(raw(
        "GraphRAG2",
        vec![
            TermCandidateEvidence::LetterDigit,
            TermCandidateEvidence::LetterDigit,
            TermCandidateEvidence::RepeatedNgram { occurrences: 2 },
            TermCandidateEvidence::RepeatedNgram { occurrences: 7 },
        ],
    ));
    let Ok(candidate) = result else {
        panic!("non-empty evidence should emit candidate");
    };

    assert_eq!(
        candidate.evidence(),
        &[
            TermCandidateEvidence::LetterDigit,
            TermCandidateEvidence::RepeatedNgram { occurrences: 7 },
        ]
    );
    assert_eq!(candidate.score().value(), 75);
}

#[test]
fn saturates_score_at_one_hundred() {
    let result = TermCandidate::try_from(raw(
        "GraphRAG2",
        vec![
            TermCandidateEvidence::AllCaps,
            TermCandidateEvidence::MixedScript,
            TermCandidateEvidence::DefinitionContext,
            TermCandidateEvidence::Emphasis,
            TermCandidateEvidence::RepeatedNgram {
                occurrences: usize::MAX,
            },
        ],
    ));
    let Ok(candidate) = result else {
        panic!("non-empty evidence should emit candidate");
    };

    assert_eq!(candidate.score().value(), 100);
}

#[test]
fn applies_kind_precedence_and_uses_unknown_only_for_empty_evidence() {
    let cases = [
        (
            vec![TermCandidateEvidence::DefinitionContext],
            TermCandidateKindHint::DefinitionLike,
        ),
        (
            vec![
                TermCandidateEvidence::MixedScript,
                TermCandidateEvidence::DefinitionContext,
            ],
            TermCandidateKindHint::DefinitionLike,
        ),
        (
            vec![
                TermCandidateEvidence::RepeatedNgram { occurrences: 2 },
                TermCandidateEvidence::MixedScript,
            ],
            TermCandidateKindHint::MixedScript,
        ),
        (
            vec![
                TermCandidateEvidence::AllCaps,
                TermCandidateEvidence::RepeatedNgram { occurrences: 2 },
            ],
            TermCandidateKindHint::RepeatedNgram,
        ),
        (
            vec![TermCandidateEvidence::AllCaps],
            TermCandidateKindHint::EnglishAcronym,
        ),
        (
            vec![TermCandidateEvidence::CamelOrPascalCase],
            TermCandidateKindHint::EnglishIdentifier,
        ),
        (
            vec![TermCandidateEvidence::LetterDigit],
            TermCandidateKindHint::EnglishIdentifier,
        ),
        (
            vec![TermCandidateEvidence::Emphasis],
            TermCandidateKindHint::ChineseCompound,
        ),
    ];

    for (evidence, expected) in cases {
        assert_eq!(TermCandidateKindHint::from_evidence(&evidence), expected);
        let result = TermCandidate::try_from(raw("GraphRAG2", evidence));
        let Ok(candidate) = result else {
            panic!("non-empty evidence should emit candidate");
        };
        assert_eq!(candidate.kind_hint(), expected);
    }
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
    fn arbitrary_permutation_does_not_change_completed_candidate_score_or_kind(
        items in prop::collection::vec((0_u64..1_000, evidence_strategy()), 1..40)
    ) {
        let evidence: Vec<_> = items.iter().map(|(_, item)| item.clone()).collect();
        let mut keyed_evidence = items;
        keyed_evidence.sort_by_key(|(priority, item)| (*priority, item.clone()));
        let permuted_evidence: Vec<_> = keyed_evidence
            .into_iter()
            .map(|(_, item)| item)
            .collect();

        let Ok(original) = TermCandidate::try_from(raw("GraphRAG2", evidence)) else {
            panic!("generated evidence is non-empty");
        };
        let Ok(permuted) = TermCandidate::try_from(raw("GraphRAG2", permuted_evidence)) else {
            panic!("generated evidence is non-empty");
        };

        prop_assert_eq!(original.evidence(), permuted.evidence());
        prop_assert_eq!(original.score(), permuted.score());
        prop_assert_eq!(original.kind_hint(), permuted.kind_hint());
    }
}
