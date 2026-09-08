use crate::latex::span::LocatedRange;

pub use super::score::CandidateScore;
use super::score::normalized_evidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermCandidate {
    surface: String,
    location: LocatedRange,
    evidence: Vec<TermCandidateEvidence>,
    kind_hint: TermCandidateKindHint,
    score: CandidateScore,
}

impl TermCandidate {
    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub const fn location(&self) -> &LocatedRange {
        &self.location
    }

    pub fn evidence(&self) -> &[TermCandidateEvidence] {
        &self.evidence
    }

    pub const fn kind_hint(&self) -> TermCandidateKindHint {
        self.kind_hint
    }

    pub const fn score(&self) -> CandidateScore {
        self.score
    }
}

impl TryFrom<RawTermOccurrence> for TermCandidate {
    type Error = ();

    fn try_from(occurrence: RawTermOccurrence) -> Result<Self, Self::Error> {
        let evidence = normalized_evidence(occurrence.evidence);
        if evidence.is_empty() {
            return Err(());
        }
        let kind_hint = TermCandidateKindHint::from_evidence(&evidence);
        let score = CandidateScore::from_evidence(&evidence);

        Ok(Self {
            surface: occurrence.surface,
            location: occurrence.location,
            evidence,
            kind_hint,
            score,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TermCandidateEvidence {
    AllCaps,
    CamelOrPascalCase,
    LetterDigit,
    MixedScript,
    DefinitionContext,
    Emphasis,
    RepeatedNgram { occurrences: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermCandidateKindHint {
    DefinitionLike,
    MixedScript,
    RepeatedNgram,
    EnglishAcronym,
    EnglishIdentifier,
    ChineseCompound,
    Unknown,
}

impl TermCandidateKindHint {
    pub fn from_evidence(evidence: &[TermCandidateEvidence]) -> Self {
        let mut has_definition_context = false;
        let mut has_mixed_script = false;
        let mut has_repeated_ngram = false;
        let mut has_all_caps = false;
        let mut has_english_identifier = false;
        let mut has_chinese_compound = false;

        for item in evidence {
            match item {
                TermCandidateEvidence::AllCaps => has_all_caps = true,
                TermCandidateEvidence::CamelOrPascalCase | TermCandidateEvidence::LetterDigit => {
                    has_english_identifier = true;
                }
                TermCandidateEvidence::MixedScript => has_mixed_script = true,
                TermCandidateEvidence::DefinitionContext => has_definition_context = true,
                TermCandidateEvidence::Emphasis => has_chinese_compound = true,
                TermCandidateEvidence::RepeatedNgram { occurrences: _ } => {
                    has_repeated_ngram = true;
                }
            }
        }

        if has_definition_context {
            return Self::DefinitionLike;
        }
        if has_mixed_script {
            return Self::MixedScript;
        }
        if has_repeated_ngram {
            return Self::RepeatedNgram;
        }
        if has_all_caps {
            return Self::EnglishAcronym;
        }
        if has_english_identifier {
            return Self::EnglishIdentifier;
        }
        if has_chinese_compound {
            return Self::ChineseCompound;
        }
        Self::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTermOccurrence {
    pub(crate) surface: String,
    pub(crate) location: LocatedRange,
    pub(crate) evidence: Vec<TermCandidateEvidence>,
}
