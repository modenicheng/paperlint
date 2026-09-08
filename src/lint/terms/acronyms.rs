use crate::{
    latex::span::LocatedRange,
    text::terminology::{extract_acronym_definitions, find_acronym_usages},
};
use std::ops::Range;

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

pub(super) struct AcronymEntries {
    pub definitions: Vec<AcronymDefinitionEntry>,
    pub usages: Vec<AcronymUsageEntry>,
}

pub(super) fn collect_acronyms(text: &str, block_index: usize) -> AcronymEntries {
    let block_definitions = extract_acronym_definitions(text);
    let definition_ranges: Vec<(String, Range<usize>)> = block_definitions
        .iter()
        .map(|definition| (definition.acronym.clone(), definition.range.clone()))
        .collect();

    let mut definitions = Vec::with_capacity(block_definitions.len());
    for definition in block_definitions {
        definitions.push(AcronymDefinitionEntry {
            acronym: definition.acronym,
            chinese: definition.chinese,
            english: definition.english,
            location: LocatedRange {
                block: block_index,
                range: definition.range,
            },
        });
    }

    let mut usages = Vec::new();
    for usage in find_acronym_usages(text) {
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

    AcronymEntries {
        definitions,
        usages,
    }
}

pub(super) fn first_definitions_in_reading_order(
    definitions: &[AcronymDefinitionEntry],
) -> Vec<AcronymDefinitionEntry> {
    let mut first_definitions: Vec<AcronymDefinitionEntry> = Vec::new();
    for definition in definitions {
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
    first_definitions
}
