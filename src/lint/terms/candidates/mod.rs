mod aggregate;
mod context;
mod context_index;
#[cfg(test)]
mod context_tests;
mod model;
#[cfg(test)]
mod model_tests;
mod ngrams;
mod prune;
#[cfg(test)]
mod prune_tests;
mod score;
mod shapes;
#[cfg(test)]
mod shapes_tests;
mod suppression;
mod text;

use crate::{
    lint::{analysis::DocumentAnalysis, terms::AcronymDefinitionEntry},
    text::lexicon::Lexicon,
};

use model::RawTermOccurrence;
pub use model::{TermCandidate, TermCandidateEvidence, TermCandidateKindHint};
pub use score::CandidateScore;

pub(crate) fn collect(
    analysis: &DocumentAnalysis,
    lexicon: &Lexicon,
    declarations: &[AcronymDefinitionEntry],
) -> Vec<TermCandidate> {
    let suppressed = suppression::SuppressedSurfaces::new(lexicon, declarations);
    let occurrences = raw_occurrences(analysis)
        .into_iter()
        .filter(|occurrence| !suppressed.contains(&occurrence.surface));
    prune::contained_repeated_only(aggregate::group(occurrences))
}

fn raw_occurrences(analysis: &DocumentAnalysis) -> Vec<RawTermOccurrence> {
    let mut occurrences = shapes::collect(analysis);
    occurrences.extend(context::collect(analysis));
    occurrences.extend(ngrams::collect(analysis));
    occurrences.sort_by_key(|occurrence| occurrence.location.position());
    occurrences
}
