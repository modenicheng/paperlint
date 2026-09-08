use super::{TermCandidate, TermCandidateEvidence};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};

type SuffixIndex<'a> = BTreeMap<&'a str, BTreeSet<&'a str>>;

pub(super) fn contained_repeated_only(candidates: Vec<TermCandidate>) -> Vec<TermCandidate> {
    contained_repeated_only_impl(candidates, || {})
}

fn contained_repeated_only_impl(
    candidates: Vec<TermCandidate>,
    mut record_inspection: impl FnMut(),
) -> Vec<TermCandidate> {
    let mut ranked_indices: Vec<_> = (0..candidates.len()).collect();
    ranked_indices.sort_by(|left, right| {
        candidate_rank(&candidates[*right]).cmp(&candidate_rank(&candidates[*left]))
    });

    let mut suffix_index = SuffixIndex::new();
    let mut pruned = vec![false; candidates.len()];
    for index in ranked_indices {
        let candidate = &candidates[index];
        if is_repeated_only(candidate)
            && has_distinct_containing_surface(
                &suffix_index,
                candidate.surface(),
                &mut record_inspection,
            )
        {
            pruned[index] = true;
        }
        index_suffixes(&mut suffix_index, candidate.surface());
    }
    drop(suffix_index);

    candidates
        .into_iter()
        .zip(pruned)
        .filter_map(|(candidate, is_pruned)| (!is_pruned).then_some(candidate))
        .collect()
}

fn is_repeated_only(candidate: &TermCandidate) -> bool {
    matches!(
        candidate.evidence(),
        [TermCandidateEvidence::RepeatedNgram { occurrences: _ }]
    )
}

fn candidate_rank(candidate: &TermCandidate) -> (u8, usize, Reverse<&str>) {
    (
        candidate.score().value(),
        candidate.surface().chars().count(),
        Reverse(candidate.surface()),
    )
}

fn has_distinct_containing_surface<'a>(
    suffix_index: &SuffixIndex<'a>,
    needle: &'a str,
    record_inspection: &mut impl FnMut(),
) -> bool {
    for (suffix, owners) in suffix_index.range(needle..) {
        record_inspection();
        if !suffix.starts_with(needle) {
            return false;
        }
        if owners.iter().any(|owner| *owner != needle) {
            return true;
        }
    }
    false
}

fn index_suffixes<'a>(suffix_index: &mut SuffixIndex<'a>, surface: &'a str) {
    for (offset, _) in surface.char_indices() {
        suffix_index
            .entry(&surface[offset..])
            .or_default()
            .insert(surface);
    }
}

#[cfg(test)]
pub(super) fn contained_repeated_only_with_inspection_count(
    candidates: Vec<TermCandidate>,
) -> (Vec<TermCandidate>, usize) {
    let mut inspections = 0;
    let retained = contained_repeated_only_impl(candidates, || inspections += 1);
    (retained, inspections)
}
