//! Shared read-only context handed to every rule for one lint run.
//!
//! [`DocumentTermRegistry`] is built exactly once per run, walks
//! [`Document::blocks`] in reading order, and owns the shared acronym and
//! lexicon analyses. Rules must reinterpret raw text through these queries
//! instead of re-deriving term knowledge on their own.

use crate::{
    config::PaperlintConfig,
    latex::parser::Document,
    latex::span::Span,
    text::{
        lexicon::{FrozenMatchers, LexemeKind, LexemeSource, Lexicon},
        terminology::{extract_acronym_definitions, find_acronym_usages},
    },
};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

/// A position in the document's reading order: block index plus byte offset
/// inside that block's logical text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadingPosition {
    pub block: usize,
    pub byte: usize,
}

/// A logical-text range inside one block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedRange {
    pub block: usize,
    pub range: Range<usize>,
}

impl LocatedRange {
    pub const fn position(&self) -> ReadingPosition {
        ReadingPosition {
            block: self.block,
            byte: self.range.start,
        }
    }
}

/// An acronym definition found in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcronymDefinitionEntry {
    pub acronym: String,
    /// Expanded form preceding the definition, when recognized.
    pub chinese: Option<String>,
    pub english: Option<String>,
    pub location: LocatedRange,
}

/// An acronym occurrence; `is_definition` marks occurrences that are part of
/// a definition pattern in the same block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcronymUsageEntry {
    pub acronym: String,
    pub location: LocatedRange,
    pub is_definition: bool,
}

/// One lexicon-resolved term occurrence in the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconOccurrence {
    /// The surface actually appearing in the text.
    pub surface: String,
    /// The canonical form of the matched lexeme.
    pub canonical: String,
    pub kind: LexemeKind,
    pub source: LexemeSource,
    /// Whether the lexeme matched this surface with case sensitivity.
    pub case_sensitive: bool,
    /// Whether the lexeme marks terms that must be explained on first use.
    pub requires_explanation: bool,
    pub location: LocatedRange,
}

/// Shared term analysis for one document, built once per lint run.
#[derive(Debug)]
pub struct DocumentTermRegistry {
    definitions: Vec<AcronymDefinitionEntry>,
    usages: Vec<AcronymUsageEntry>,
    first_definitions: Vec<AcronymDefinitionEntry>,
    lexicon_occurrences: Vec<LexiconOccurrence>,
    first_lexicon_occurrences: HashMap<String, usize>,
}

impl DocumentTermRegistry {
    /// Walk `document.blocks` in reading order and collect the acronym and
    /// lexicon analyses.
    pub fn new(document: &Document, lexicon: &Lexicon) -> Self {
        let mut definitions = Vec::new();
        let mut usages = Vec::new();
        let mut lexicon_occurrences = Vec::new();
        // Frozen scan matchers are compiled once per run, not per block.
        let frozen_matchers = lexicon.matchers();

        for (block_index, block) in document.blocks.iter().enumerate() {
            let block_definitions = extract_acronym_definitions(&block.text);
            let definition_ranges: Vec<(String, Range<usize>)> = block_definitions
                .iter()
                .map(|definition| (definition.acronym.clone(), definition.range.clone()))
                .collect();

            definitions.reserve(block_definitions.len());
            let mut converted_definitions = Vec::with_capacity(block_definitions.len());
            for definition in block_definitions {
                converted_definitions.push(AcronymDefinitionEntry {
                    acronym: definition.acronym,
                    chinese: definition.chinese,
                    english: definition.english,
                    location: LocatedRange {
                        block: block_index,
                        range: definition.range,
                    },
                });
            }
            definitions.extend(converted_definitions);

            for usage in find_acronym_usages(&block.text) {
                let is_definition = definition_ranges
                    .iter()
                    .any(|(acronym, range)| *acronym == usage.acronym && *range == usage.range);
                usages.push(AcronymUsageEntry {
                    acronym: usage.acronym,
                    location: LocatedRange {
                        block: block_index,
                        range: usage.range,
                    },
                    is_definition,
                });
            }

            collect_lexicon_occurrences(
                &block.text,
                block_index,
                lexicon,
                frozen_matchers.as_deref(),
                &mut lexicon_occurrences,
            );
        }

        lexicon_occurrences.sort_by(|left, right| {
            left.location
                .position()
                .cmp(&right.location.position())
                .then(left.location.range.end.cmp(&right.location.range.end))
                .then(left.canonical.cmp(&right.canonical))
        });
        let mut first_lexicon_occurrences: HashMap<String, usize> = HashMap::new();
        for (index, occurrence) in lexicon_occurrences.iter().enumerate() {
            first_lexicon_occurrences
                .entry(occurrence.canonical.clone())
                .or_insert(index);
        }

        // Earliest definition per acronym, in reading order.
        let mut first_definitions: Vec<AcronymDefinitionEntry> = Vec::new();
        for definition in &definitions {
            if let Some(existing) = first_definitions
                .iter_mut()
                .find(|existing| existing.acronym == definition.acronym)
            {
                if definition.location.position() < existing.location.position() {
                    *existing = definition.clone();
                }
            } else {
                first_definitions.push(definition.clone());
            }
        }
        first_definitions.sort_by_key(|definition| definition.location.position());

        Self {
            definitions,
            usages,
            first_definitions,
            lexicon_occurrences,
            first_lexicon_occurrences,
        }
    }

    /// All acronym definitions in reading order.
    pub fn definitions(&self) -> &[AcronymDefinitionEntry] {
        &self.definitions
    }

    /// All acronym occurrences in reading order, definition occurrences
    /// included and flagged.
    pub fn usages(&self) -> &[AcronymUsageEntry] {
        &self.usages
    }

    /// The earliest definition of `acronym`, if any.
    pub fn first_definition(&self, acronym: &str) -> Option<&AcronymDefinitionEntry> {
        self.first_definitions
            .iter()
            .find(|definition| definition.acronym == acronym)
    }

    /// Earliest definition per acronym, in reading order.
    pub fn first_definitions(&self) -> &[AcronymDefinitionEntry] {
        &self.first_definitions
    }

    /// Non-definition occurrences of `acronym` strictly after `position`, in
    /// reading order.
    pub fn usages_after(
        &self,
        acronym: &str,
        position: ReadingPosition,
    ) -> impl Iterator<Item = &AcronymUsageEntry> {
        self.usages.iter().filter(move |usage| {
            !usage.is_definition && usage.acronym == acronym && usage.location.position() > position
        })
    }

    /// All lexicon-resolved occurrences in reading order.
    pub fn lexicon_occurrences(&self) -> &[LexiconOccurrence] {
        &self.lexicon_occurrences
    }

    /// Lexicon-resolved occurrences of one canonical form, in reading order.
    pub fn lexicon_occurrences_of(
        &self,
        canonical: &str,
    ) -> impl Iterator<Item = &LexiconOccurrence> {
        self.lexicon_occurrences
            .iter()
            .filter(move |occurrence| occurrence.canonical == canonical)
    }

    /// The first lexicon-resolved occurrence of `canonical` in reading order.
    pub fn first_lexicon_occurrence(&self, canonical: &str) -> Option<&LexiconOccurrence> {
        self.first_lexicon_occurrences
            .get(canonical)
            .map(|&index| &self.lexicon_occurrences[index])
    }
}

/// Collect lexicon occurrences for one block using the lexicon's frozen
/// Aho-Corasick matchers, compiled once per [`DocumentTermRegistry`] and
/// shared across blocks. Standard match kind plus `find_overlapping_iter`
/// reports every overlapping hit, exactly like the per-key `match_indices`
/// scan this replaces, at one automaton pass per block instead of one scan
/// per key. Two invariants hold:
///
/// - Every hit is re-verified through [`Lexicon::lookup`], the single
///   source of surface attribution: a span is recorded only when the frozen
///   index still resolves the matched surface to the lexeme whose key found
///   it. Two lexemes can therefore never both claim the same span, even when
///   an alias equals another entry's canonical form or a folded key is
///   contested.
/// - Occurrences are deduplicated on `(block, start, end, canonical)` so a
///   surface equal to its own alias is counted once; distinct spans may
///   still nest, e.g. "neural network" inside "neural networks".
fn collect_lexicon_occurrences(
    text: &str,
    block_index: usize,
    lexicon: &Lexicon,
    matchers: Option<&FrozenMatchers>,
    occurrences: &mut Vec<LexiconOccurrence>,
) {
    // A lexicon without scan keys compiles no matchers; there is nothing
    // to record.
    let Some(matchers) = matchers else {
        return;
    };
    let lowered = text.to_ascii_lowercase();
    let mut seen_spans: HashSet<(usize, usize, usize, &str)> = HashSet::new();
    for matched in matchers.exact.find_overlapping_iter(text) {
        collect_match(
            text,
            block_index,
            matched.start()..matched.end(),
            &matchers.exact_key(&matched).canonical,
            lexicon,
            &mut seen_spans,
            occurrences,
        );
    }
    for matched in matchers.folded.find_overlapping_iter(&lowered) {
        collect_match(
            text,
            block_index,
            matched.start()..matched.end(),
            &matchers.folded_key(&matched).canonical,
            lexicon,
            &mut seen_spans,
            occurrences,
        );
    }
}

fn collect_match<'a>(
    text: &str,
    block_index: usize,
    range: Range<usize>,
    expected_canonical: &str,
    lexicon: &'a Lexicon,
    seen_spans: &mut HashSet<(usize, usize, usize, &'a str)>,
    occurrences: &mut Vec<LexiconOccurrence>,
) {
    if !is_bounded_match(text, range.start, range.end) {
        return;
    }
    let surface = &text[range.clone()];
    let Some(lexeme) = lexicon.lookup(surface) else {
        return;
    };
    if lexeme.canonical != expected_canonical {
        return;
    }
    let span_key = (
        block_index,
        range.start,
        range.end,
        lexeme.canonical.as_str(),
    );
    if !seen_spans.insert(span_key) {
        return;
    }
    occurrences.push(LexiconOccurrence {
        surface: surface.to_string(),
        canonical: lexeme.canonical.clone(),
        kind: lexeme.kind,
        source: lexeme.source,
        case_sensitive: lexeme.case_sensitive,
        requires_explanation: lexeme.requires_explanation,
        location: LocatedRange {
            block: block_index,
            range,
        },
    });
}

/// Reject matches glued to ASCII identifier bytes: `AIM` must not match
/// `AI`, and `FOO_CNN_BAR` must not match `CNN`. Underscore counts as an
/// identifier byte because it keeps ASCII identifiers together, while
/// non-ASCII neighbors never block a match, so CJK surfaces match freely
/// (e.g. `基于CNN的` still matches `CNN`).
fn is_bounded_match(text: &str, start: usize, end: usize) -> bool {
    let bytes = text.as_bytes();
    let identifier_byte = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    let before_ok = start == 0 || !identifier_byte(bytes[start - 1]);
    let after_ok = end >= bytes.len() || !identifier_byte(bytes[end]);
    before_ok && after_ok
}

/// Everything a rule needs for one lint run. Built once per document by the
/// engine and shared read-only across all rules.
pub struct LintContext<'a> {
    document: &'a Document,
    config: &'a PaperlintConfig,
    lexicon: &'a Lexicon,
    registry: DocumentTermRegistry,
}

impl<'a> LintContext<'a> {
    pub fn new(document: &'a Document, config: &'a PaperlintConfig, lexicon: &'a Lexicon) -> Self {
        Self {
            document,
            config,
            lexicon,
            registry: DocumentTermRegistry::new(document, lexicon),
        }
    }

    pub const fn document(&self) -> &'a Document {
        self.document
    }

    pub const fn config(&self) -> &'a PaperlintConfig {
        self.config
    }

    pub const fn lexicon(&self) -> &'a Lexicon {
        self.lexicon
    }

    pub const fn registry(&self) -> &DocumentTermRegistry {
        &self.registry
    }

    /// Map a block-relative logical range to its LaTeX source span.
    pub fn span(&self, block: usize, range: Range<usize>) -> Option<Span> {
        let block = self.document.blocks.get(block)?;
        self.document.source_span(block, range)
    }

    /// Map a located range to its LaTeX source span.
    pub fn span_of(&self, located: &LocatedRange) -> Option<Span> {
        self.span(located.block, located.range.clone())
    }
}
