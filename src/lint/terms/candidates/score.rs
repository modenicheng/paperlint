use super::model::TermCandidateEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidateScore(u8);

impl CandidateScore {
    const MAX: u8 = 100;

    pub const fn value(self) -> u8 {
        self.0
    }

    pub fn from_evidence(evidence: &[TermCandidateEvidence]) -> Self {
        let mut score = 0_u8;
        for item in normalized_evidence(evidence.iter().cloned()) {
            score = score.saturating_add(item.weight()).min(Self::MAX);
        }
        Self(score)
    }
}

impl TermCandidateEvidence {
    const fn weight(&self) -> u8 {
        match self {
            Self::AllCaps => 40,
            Self::CamelOrPascalCase => 35,
            Self::LetterDigit => 35,
            Self::MixedScript => 45,
            Self::DefinitionContext => 35,
            Self::Emphasis => 25,
            Self::RepeatedNgram { occurrences } => repeated_ngram_weight(*occurrences),
        }
    }
}

pub(super) fn normalized_evidence(
    evidence: impl IntoIterator<Item = TermCandidateEvidence>,
) -> Vec<TermCandidateEvidence> {
    let mut has_all_caps = false;
    let mut has_camel_or_pascal_case = false;
    let mut has_letter_digit = false;
    let mut has_mixed_script = false;
    let mut has_definition_context = false;
    let mut has_emphasis = false;
    let mut repeated_ngram_occurrences: Option<usize> = None;

    for item in evidence {
        match item {
            TermCandidateEvidence::AllCaps => has_all_caps = true,
            TermCandidateEvidence::CamelOrPascalCase => has_camel_or_pascal_case = true,
            TermCandidateEvidence::LetterDigit => has_letter_digit = true,
            TermCandidateEvidence::MixedScript => has_mixed_script = true,
            TermCandidateEvidence::DefinitionContext => has_definition_context = true,
            TermCandidateEvidence::Emphasis => has_emphasis = true,
            TermCandidateEvidence::RepeatedNgram { occurrences } => {
                repeated_ngram_occurrences = Some(
                    repeated_ngram_occurrences
                        .map_or(occurrences, |existing| existing.max(occurrences)),
                );
            }
        }
    }

    let mut normalized = Vec::new();
    if has_all_caps {
        normalized.push(TermCandidateEvidence::AllCaps);
    }
    if has_camel_or_pascal_case {
        normalized.push(TermCandidateEvidence::CamelOrPascalCase);
    }
    if has_letter_digit {
        normalized.push(TermCandidateEvidence::LetterDigit);
    }
    if has_mixed_script {
        normalized.push(TermCandidateEvidence::MixedScript);
    }
    if has_definition_context {
        normalized.push(TermCandidateEvidence::DefinitionContext);
    }
    if has_emphasis {
        normalized.push(TermCandidateEvidence::Emphasis);
    }
    if let Some(occurrences) = repeated_ngram_occurrences {
        normalized.push(TermCandidateEvidence::RepeatedNgram { occurrences });
    }
    normalized
}

const fn repeated_ngram_weight(occurrences: usize) -> u8 {
    let bonus_steps = if occurrences < 4 { occurrences } else { 4 };
    let mut bonus = 0_u8;
    let mut step = 0_usize;
    while step < bonus_steps {
        bonus += 5;
        step += 1;
    }
    20 + bonus
}
