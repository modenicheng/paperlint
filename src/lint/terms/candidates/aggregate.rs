use super::{RawTermOccurrence, TermCandidate, TermCandidateEvidence};
use crate::latex::span::LocatedRange;
use std::collections::BTreeMap;

pub(super) fn group(
    occurrences: impl IntoIterator<Item = RawTermOccurrence>,
) -> Vec<TermCandidate> {
    let mut grouped = BTreeMap::new();
    for occurrence in occurrences {
        grouped
            .entry(occurrence.surface.clone())
            .or_insert_with(|| CandidateGroup::new(occurrence.surface.clone()))
            .push(occurrence);
    }
    let mut candidates: Vec<_> = grouped
        .into_values()
        .filter_map(CandidateGroup::into_candidate)
        .collect();
    candidates.sort_by(candidate_order);
    candidates
}

struct CandidateGroup {
    surface: String,
    first_location: Option<LocatedRange>,
    evidence: Vec<TermCandidateEvidence>,
}

impl CandidateGroup {
    fn new(surface: String) -> Self {
        Self {
            surface,
            first_location: None,
            evidence: Vec::new(),
        }
    }

    fn push(&mut self, occurrence: RawTermOccurrence) {
        if self
            .first_location
            .as_ref()
            .is_none_or(|location| occurrence.location.position() < location.position())
        {
            self.first_location = Some(occurrence.location);
        }
        self.evidence.extend(occurrence.evidence);
    }

    fn into_candidate(self) -> Option<TermCandidate> {
        let location = self.first_location?;
        TermCandidate::try_from(RawTermOccurrence {
            surface: self.surface,
            location,
            evidence: self.evidence,
        })
        .ok()
    }
}

fn candidate_order(left: &TermCandidate, right: &TermCandidate) -> std::cmp::Ordering {
    left.location()
        .position()
        .cmp(&right.location().position())
        .then(left.surface().cmp(right.surface()))
}
