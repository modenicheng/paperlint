use super::{
    model::{RawTermOccurrence, TermCandidate, TermCandidateEvidence},
    prune::{contained_repeated_only, contained_repeated_only_with_inspection_count},
};
use crate::latex::span::LocatedRange;
use std::cmp::Reverse;

#[test]
fn candidate_pair_work_is_bounded_when_surfaces_do_not_overlap() {
    // Given: hundreds of repeated-only candidates with no containment relation.
    let candidates: Vec<_> = (0..512)
        .map(|index| repeated(&format!("术语{index:04}"), 2, index))
        .collect();

    // When: contained repeated-only candidates are pruned.
    let (retained, inspections) = contained_repeated_only_with_inspection_count(candidates);

    // Then: work is bounded per candidate instead of inspecting every pair.
    assert_eq!(retained.len(), 512);
    assert!(
        inspections <= 512 * 4,
        "expected indexed pruning, observed {inspections} candidate-pair inspections"
    );
}

#[test]
fn optimized_pruning_matches_reference_for_every_adversarial_permutation() {
    // Given: Unicode containment, unequal scores, and a non-repeated candidate.
    let candidates = vec![
        repeated("语义", 3, 0),
        repeated("跨模态语义", 3, 1),
        repeated("模型", 4, 2),
        candidate(
            "模型架构",
            vec![TermCandidateEvidence::DefinitionContext],
            3,
        ),
        candidate("网络", vec![TermCandidateEvidence::Emphasis], 4),
        repeated("神经网络", 2, 5),
    ];
    let mut permutations = Vec::new();
    collect_permutations(&mut candidates.clone(), 0, &mut permutations);

    // When: every ordering is passed through optimized and reference pruning.
    // Then: all fields and the original retained order are exactly equivalent.
    for permutation in permutations {
        assert_eq!(
            contained_repeated_only(permutation.clone()),
            reference_prune(permutation)
        );
    }
}

#[test]
fn optimized_pruning_matches_reference_for_overlapping_ascii_false_leads() {
    // Given: overlapping substrings, equal scores, and surfaces that only share fragments.
    let candidates = vec![
        repeated("aba", 2, 0),
        repeated("bab", 2, 1),
        repeated("zabaz", 2, 2),
        repeated("zzbab", 2, 3),
        repeated("abx", 2, 4),
        candidate("prefix-abx", vec![TermCandidateEvidence::AllCaps], 5),
    ];
    let orderings = [
        candidates.clone(),
        candidates.iter().rev().cloned().collect(),
        vec![
            candidates[2].clone(),
            candidates[0].clone(),
            candidates[5].clone(),
            candidates[3].clone(),
            candidates[1].clone(),
            candidates[4].clone(),
        ],
    ];

    // When/Then: optimized pruning agrees with the exhaustive reference.
    for ordering in orderings {
        assert_eq!(
            contained_repeated_only(ordering.clone()),
            reference_prune(ordering)
        );
    }
}

fn candidate(surface: &str, evidence: Vec<TermCandidateEvidence>, start: usize) -> TermCandidate {
    TermCandidate::try_from(RawTermOccurrence {
        surface: surface.to_string(),
        location: LocatedRange {
            block: 0,
            range: start..start + surface.len(),
        },
        evidence,
    })
    .expect("test candidates always carry evidence")
}

fn repeated(surface: &str, occurrences: usize, start: usize) -> TermCandidate {
    candidate(
        surface,
        vec![TermCandidateEvidence::RepeatedNgram { occurrences }],
        start,
    )
}

fn reference_prune(candidates: Vec<TermCandidate>) -> Vec<TermCandidate> {
    candidates
        .iter()
        .filter(|candidate| !reference_is_pruned(candidate, &candidates))
        .cloned()
        .collect()
}

fn reference_is_pruned(candidate: &TermCandidate, candidates: &[TermCandidate]) -> bool {
    if !matches!(
        candidate.evidence(),
        [TermCandidateEvidence::RepeatedNgram { occurrences: _ }]
    ) {
        return false;
    }
    candidates.iter().any(|other| {
        candidate.surface() != other.surface()
            && other.surface().contains(candidate.surface())
            && reference_rank(other) >= reference_rank(candidate)
    })
}

fn reference_rank(candidate: &TermCandidate) -> (u8, usize, Reverse<&str>) {
    (
        candidate.score().value(),
        candidate.surface().chars().count(),
        Reverse(candidate.surface()),
    )
}

fn collect_permutations<T: Clone>(items: &mut [T], start: usize, output: &mut Vec<Vec<T>>) {
    if start == items.len() {
        output.push(items.to_vec());
        return;
    }
    for index in start..items.len() {
        items.swap(start, index);
        collect_permutations(items, start + 1, output);
        items.swap(start, index);
    }
}
