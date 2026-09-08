use super::{DocumentTermRegistry, TermCandidateEvidence, TermCandidateKindHint};
use crate::{
    config::DefaultConfig,
    latex::{parser, span::LocatedRange},
    lint::analysis::{DocumentAnalysis, ParagraphUnit, SentenceUnit},
    nlp::{PosTag, Token},
    text::{Language, lexicon::Lexicon},
};

const PHRASE: &str = "跨模态语义蒸馏网络";

#[test]
fn registry_groups_repeated_phrase_and_prunes_equal_score_subgrams() {
    let config = DefaultConfig::load();
    let text = format!("{PHRASE}。{PHRASE}。");
    let document = parser::parse_stdin(text.clone(), &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let second_start = PHRASE.len() + "。".len();
    let analysis = analysis(vec![
        nominal_sentence(0, 0, &["跨模态", "语义", "蒸馏", "网络"]),
        nominal_sentence(0, second_start, &["跨模态", "语义", "蒸馏", "网络"]),
    ]);

    let registry = DocumentTermRegistry::with_analysis(&document, &lexicon, &analysis);
    let matches: Vec<_> = registry
        .term_candidates()
        .iter()
        .filter(|candidate| candidate.surface() == PHRASE)
        .collect();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location().range, 0..PHRASE.len());
    assert_eq!(
        matches[0].evidence(),
        [TermCandidateEvidence::RepeatedNgram { occurrences: 2 }]
    );
    assert_eq!(matches[0].score().value(), 30);
    assert_eq!(matches[0].kind_hint(), TermCandidateKindHint::RepeatedNgram);
    assert!(!registry.term_candidates().iter().any(|candidate| {
        candidate.surface() != PHRASE
            && PHRASE.contains(candidate.surface())
            && matches!(
                candidate.evidence(),
                [TermCandidateEvidence::RepeatedNgram { occurrences: _ }]
            )
    }));
}

#[test]
fn pruning_prefers_score_before_containing_surface_length() {
    let config = DefaultConfig::load();
    let document = parser::parse_stdin(String::new(), &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let mut sentences = Vec::new();
    let mut start = 0;
    for _ in 0..2 {
        sentences.push(nominal_sentence(
            0,
            start,
            &["跨模态", "语义", "蒸馏", "网络"],
        ));
        start += PHRASE.len() + "。".len();
    }
    for _ in 0..2 {
        sentences.push(nominal_sentence(0, start, &["跨模态", "语义"]));
        start += "跨模态语义。".len();
    }
    let analysis = analysis(sentences);

    let registry = DocumentTermRegistry::with_analysis(&document, &lexicon, &analysis);
    let prefix = registry
        .term_candidates()
        .iter()
        .find(|candidate| candidate.surface() == "跨模态语义")
        .unwrap();
    let full = registry
        .term_candidates()
        .iter()
        .find(|candidate| candidate.surface() == PHRASE)
        .unwrap();

    assert_eq!(prefix.score().value(), 40);
    assert_eq!(full.score().value(), 30);
}

#[test]
fn registry_retains_logical_candidate_without_source_mapping() {
    let config = DefaultConfig::load();
    let document = parser::parse_stdin(String::new(), &config.latex).unwrap();
    let lexicon = Lexicon::from_config(&config);
    let sentence = SentenceUnit {
        text: "GraphRAG2".to_string(),
        language: Language::English,
        location: LocatedRange {
            block: 99,
            range: 100..109,
        },
        tokens: Vec::new(),
    };
    let analysis = DocumentAnalysis::from_paragraphs_for_tests(vec![ParagraphUnit {
        block: 99,
        location: sentence.location.clone(),
        sentences: vec![sentence],
    }]);

    let registry = DocumentTermRegistry::with_analysis(&document, &lexicon, &analysis);
    let candidate = registry
        .term_candidates()
        .iter()
        .find(|candidate| candidate.surface() == "GraphRAG2")
        .unwrap();

    assert_eq!(candidate.location().block, 99);
    assert_eq!(candidate.location().range, 100..109);
}

fn analysis(sentences: Vec<SentenceUnit>) -> DocumentAnalysis {
    let end = sentences
        .last()
        .map_or(0, |sentence| sentence.location.range.end);
    DocumentAnalysis::from_paragraphs_for_tests(vec![ParagraphUnit {
        block: 0,
        location: LocatedRange {
            block: 0,
            range: 0..end,
        },
        sentences,
    }])
}

fn nominal_sentence(block: usize, start: usize, surfaces: &[&str]) -> SentenceUnit {
    let body = surfaces.concat();
    let text = format!("{body}。");
    let mut offset = start;
    let tokens = surfaces
        .iter()
        .map(|surface| {
            let token = Token {
                surface: (*surface).to_string(),
                pos: PosTag::Noun,
                location: LocatedRange {
                    block,
                    range: offset..offset + surface.len(),
                },
            };
            offset += surface.len();
            token
        })
        .collect();
    SentenceUnit {
        text,
        language: Language::Chinese,
        location: LocatedRange {
            block,
            range: start..start + body.len() + "。".len(),
        },
        tokens,
    }
}
