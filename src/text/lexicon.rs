//! Shared term lexicon for semantic lint rules.
//!
//! The lexicon merges three layers, later layers overriding earlier ones per
//! canonical form: built-in entries, workspace config entries
//! (`[[lexicon.entries]]`), and acronyms stacked from `ACR001.ignore` so the
//! legacy ignore list keeps working. Rules resolve surfaces through
//! [`Lexicon::lookup`] instead of re-deriving term knowledge on their own.
//!
//! # Surface attribution (freeze policy)
//!
//! The index built by `rebuild_index` is the single source of truth for
//! which lexeme owns a surface:
//!
//! - Config entries are trimmed while deserializing, and empty or
//!   whitespace-only canonical/alias values are rejected as configuration
//!   errors; the index additionally drops any empty surface so an empty
//!   needle can never reach `match_indices`.
//! - Case-sensitive lexemes register exact keys, case-insensitive ones
//!   register ASCII-folded keys. [`Lexicon::lookup`] tries exact keys
//!   first, so an exact key shadows a folded key over the same letters.
//! - When two lexemes claim the same key in the same map, the lexeme
//!   inserted later wins: workspace entries override built-ins, and within
//!   the workspace the last entry in config order wins.
//! - Occurrence collection scans these frozen keys and re-verifies every
//!   hit through [`Lexicon::lookup`] so one source span is never
//!   attributed to two lexemes; see
//!   [`crate::lint::context::DocumentTermRegistry`].

use crate::config::{LexiconEntryConfig, PaperlintConfig};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// The semantic class of a lexeme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LexemeKind {
    Common,
    Term,
    Acronym,
    ProperNoun,
    Unit,
    Symbol,
}

impl LexemeKind {
    /// Whether surface matching for this kind keeps letter case by default.
    pub const fn default_case_sensitive(self) -> bool {
        match self {
            Self::Common | Self::Term => false,
            Self::Acronym | Self::ProperNoun | Self::Unit | Self::Symbol => true,
        }
    }
}

/// Which layer a lexeme came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LexemeSource {
    /// Ships with paperlint.
    Builtin,
    /// Declared in workspace config (`[[lexicon.entries]]`).
    Workspace,
    /// Stacked from `ACR001.ignore` for backward compatibility.
    AcronymIgnore,
}

/// One entry of the shared lexicon.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lexeme {
    /// Canonical spelling used as the entry identity.
    pub canonical: String,
    /// Additional surfaces resolving to this lexeme.
    pub aliases: Vec<String>,
    pub kind: LexemeKind,
    pub source: LexemeSource,
    pub case_sensitive: bool,
    pub requires_explanation: bool,
}

impl Lexeme {
    /// Materialize a workspace lexeme from its config entry. Config entries
    /// are already trimmed by deserialization, so surfaces are never empty.
    pub(crate) fn from_entry(entry: &LexiconEntryConfig) -> Self {
        Self {
            canonical: entry.canonical.clone(),
            aliases: entry.aliases.clone(),
            kind: entry.kind,
            source: LexemeSource::Workspace,
            case_sensitive: entry
                .case_sensitive
                .unwrap_or(entry.kind.default_case_sensitive()),
            requires_explanation: entry.requires_explanation,
        }
    }
}

/// One surface to scan for during occurrence collection. Keys are derived
/// from the frozen lookup index rather than from lexeme fields, so scan
/// attribution and [`Lexicon::lookup`] can never disagree.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScanKey<'a> {
    /// The byte pattern to search for: the exact surface for case-sensitive
    /// lexemes, or its ASCII-folded form otherwise.
    pub(crate) needle: &'a str,
    /// The canonical form the index attributes this surface to.
    pub(crate) canonical: &'a str,
    /// Whether `needle` must be searched in ASCII-lowercased text.
    pub(crate) folded: bool,
}

/// A lookup table from term surfaces to their canonical lexemes.
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    lexemes: Vec<Lexeme>,
    /// Canonical form -> index into `lexemes`, keeping lookups O(1).
    by_canonical: HashMap<String, usize>,
    /// Exact surface key -> canonical, for case-sensitive entries.
    exact: HashMap<String, String>,
    /// ASCII-lowercased surface key -> canonical, for case-insensitive entries.
    folded: HashMap<String, String>,
}

impl Lexicon {
    /// Built-in entries; today only the default `ACR001.ignore` acronyms so
    /// migration keeps behavior unchanged.
    pub(crate) fn builtin() -> Self {
        let mut lexicon = Self::default();
        for acronym in ["AI", "CPU", "GPU", "API"] {
            lexicon.insert(Lexeme {
                canonical: acronym.to_string(),
                aliases: Vec::new(),
                kind: LexemeKind::Acronym,
                source: LexemeSource::Builtin,
                case_sensitive: LexemeKind::Acronym.default_case_sensitive(),
                requires_explanation: false,
            });
        }
        lexicon
    }

    /// Build the run lexicon: built-ins, then workspace entries (overriding
    /// built-ins with the same canonical), then `ACR001.ignore` acronyms
    /// stacked as known acronyms when not already present. Batch inserts
    /// rebuild the index once per layer, so large configs stay near O(n).
    pub fn from_config(config: &PaperlintConfig) -> Self {
        let mut lexicon = Self::builtin();
        lexicon.overlay_workspace(config.lexicon.entries.iter().map(Lexeme::from_entry));
        lexicon.ensure_known_acronyms(config.rules.acr001.ignore.iter().map(String::as_str));
        lexicon
    }

    /// Overlay workspace entries with a single index rebuild. Entries with a
    /// canonical form already present replace the earlier lexeme entirely;
    /// when the batch itself repeats a canonical form, the later entry in
    /// config order wins.
    pub(crate) fn overlay_workspace(&mut self, entries: impl IntoIterator<Item = Lexeme>) {
        self.insert_all(entries);
    }

    /// Insert one lexeme, replacing any existing lexeme with the same
    /// canonical form.
    pub(crate) fn insert(&mut self, lexeme: Lexeme) {
        self.insert_all([lexeme]);
    }

    /// Stack `ACR001.ignore` acronyms as known acronyms with one index
    /// rebuild. Surfaces already resolving to some lexeme are kept as-is;
    /// empty entries are ignored so they never produce empty scan keys.
    pub(crate) fn ensure_known_acronyms<'a>(
        &mut self,
        acronyms: impl IntoIterator<Item = &'a str>,
    ) {
        let mut fresh = Vec::new();
        for acronym in acronyms {
            if acronym.trim().is_empty() || self.lookup(acronym).is_some() {
                continue;
            }
            fresh.push(Lexeme {
                canonical: acronym.to_string(),
                aliases: Vec::new(),
                kind: LexemeKind::Acronym,
                source: LexemeSource::AcronymIgnore,
                case_sensitive: LexemeKind::Acronym.default_case_sensitive(),
                requires_explanation: false,
            });
        }
        self.insert_all(fresh);
    }

    /// Insert a batch of lexemes with one index rebuild. Afterwards each
    /// canonical form appears at most once: later batch entries override
    /// earlier ones, and every pre-existing lexeme with the same canonical
    /// form is replaced.
    fn insert_all(&mut self, entries: impl IntoIterator<Item = Lexeme>) {
        let mut positions: HashMap<String, usize> = HashMap::new();
        let mut batch: Vec<Lexeme> = Vec::new();
        for lexeme in entries {
            match positions.get(&lexeme.canonical) {
                Some(&position) => batch[position] = lexeme,
                None => {
                    positions.insert(lexeme.canonical.clone(), batch.len());
                    batch.push(lexeme);
                }
            }
        }
        if batch.is_empty() {
            return;
        }
        let replaced: HashSet<&str> = batch
            .iter()
            .map(|lexeme| lexeme.canonical.as_str())
            .collect();
        self.lexemes
            .retain(|existing| !replaced.contains(existing.canonical.as_str()));
        self.lexemes.append(&mut batch);
        self.rebuild_index();
    }

    /// Resolve a surface to its lexeme, trying an exact match first and then
    /// an ASCII case-folded match.
    pub fn lookup(&self, surface: &str) -> Option<&Lexeme> {
        let canonical = self
            .exact
            .get(surface)
            .or_else(|| self.folded.get(&surface.to_ascii_lowercase()))?;
        let index = *self.by_canonical.get(canonical)?;
        self.lexemes.get(index)
    }

    /// Iterate lexemes in insertion order (built-ins first).
    pub fn entries(&self) -> impl Iterator<Item = &Lexeme> {
        self.lexemes.iter()
    }

    /// Surfaces to scan for occurrence collection, derived from the frozen
    /// index: exact keys first, then folded keys. Every key already reflects
    /// the freeze policy (exact over folded, later inserts over earlier),
    /// so scanning these keys cannot attribute one span twice.
    pub(crate) fn scan_keys(&self) -> impl Iterator<Item = ScanKey<'_>> {
        self.exact
            .iter()
            .map(|(needle, canonical)| ScanKey {
                needle: needle.as_str(),
                canonical: canonical.as_str(),
                folded: false,
            })
            .chain(self.folded.iter().map(|(needle, canonical)| ScanKey {
                needle: needle.as_str(),
                canonical: canonical.as_str(),
                folded: true,
            }))
    }

    fn rebuild_index(&mut self) {
        self.by_canonical.clear();
        self.exact.clear();
        self.folded.clear();
        for (index, lexeme) in self.lexemes.iter().enumerate() {
            self.by_canonical.insert(lexeme.canonical.clone(), index);
            for surface in std::iter::once(&lexeme.canonical).chain(&lexeme.aliases) {
                // Parsed config entries reject empty surfaces, and other
                // construction paths drop them here: an empty needle would
                // match_indices at every position and flood occurrences.
                if surface.trim().is_empty() {
                    continue;
                }
                if lexeme.case_sensitive {
                    self.exact.insert(surface.clone(), lexeme.canonical.clone());
                } else {
                    self.folded
                        .insert(surface.to_ascii_lowercase(), lexeme.canonical.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lexeme(canonical: &str, kind: LexemeKind, case_sensitive: bool) -> Lexeme {
        Lexeme {
            canonical: canonical.to_string(),
            aliases: Vec::new(),
            kind,
            source: LexemeSource::Workspace,
            case_sensitive,
            requires_explanation: false,
        }
    }

    #[test]
    fn builtin_lexicon_holds_default_acr001_ignore_acronyms() {
        let lexicon = Lexicon::builtin();
        for acronym in ["AI", "CPU", "GPU", "API"] {
            let found = lexicon.lookup(acronym).expect(acronym);
            assert_eq!(found.kind, LexemeKind::Acronym, "{acronym}");
            assert_eq!(found.source, LexemeSource::Builtin, "{acronym}");
            assert!(!found.requires_explanation, "{acronym}");
        }
        assert_eq!(lexicon.entries().collect::<Vec<_>>().len(), 4);
        assert!(lexicon.lookup("LLM").is_none());
    }

    #[test]
    fn workspace_entry_overrides_builtin_with_same_canonical() {
        let mut lexicon = Lexicon::builtin();
        let mut entry = lexeme("AI", LexemeKind::Term, false);
        entry.source = LexemeSource::Workspace;
        entry.requires_explanation = true;
        lexicon.insert(entry);

        let found = lexicon.lookup("AI").expect("overridden entry");
        assert_eq!(found.kind, LexemeKind::Term);
        assert_eq!(found.source, LexemeSource::Workspace);
        assert!(found.requires_explanation);
        assert_eq!(
            lexicon
                .entries()
                .filter(|lexeme| lexeme.canonical == "AI")
                .count(),
            1
        );
        // The override replaced the case-sensitive built-in keys entirely.
        assert!(lexicon.lookup("ai").is_some());
    }

    #[test]
    fn aliases_resolve_to_their_canonical_lexeme() {
        let mut lexicon = Lexicon::default();
        let mut entry = lexeme("dataset", LexemeKind::Common, false);
        entry.aliases = vec!["data set".to_string()];
        lexicon.insert(entry);

        for surface in ["dataset", "Dataset", "data set", "DATA SET"] {
            let found = lexicon.lookup(surface).expect(surface);
            assert_eq!(found.canonical, "dataset");
        }
        assert!(lexicon.lookup("datasets").is_none());
    }

    #[test]
    fn case_sensitive_lookup_is_exact_only() {
        let mut lexicon = Lexicon::default();
        let mut entry = lexeme("GitHub", LexemeKind::ProperNoun, true);
        entry.aliases = vec!["Github".to_string()];
        lexicon.insert(entry);

        assert!(lexicon.lookup("GitHub").is_some());
        assert!(lexicon.lookup("Github").is_some());
        assert!(lexicon.lookup("github").is_none());
        assert!(lexicon.lookup("GITHUB").is_none());
    }

    #[test]
    fn ensure_known_acronym_adds_missing_and_keeps_existing() {
        let mut lexicon = Lexicon::builtin();
        lexicon.ensure_known_acronyms(["XYZ", "AI"]);

        let added = lexicon.lookup("XYZ").expect("stacked acronym");
        assert_eq!(added.kind, LexemeKind::Acronym);
        assert_eq!(added.source, LexemeSource::AcronymIgnore);
        assert_eq!(
            lexicon.lookup("AI").expect("builtin kept").source,
            LexemeSource::Builtin
        );
        assert_eq!(lexicon.entries().count(), 5);
    }

    #[test]
    fn entries_iterate_in_insertion_order() {
        let mut lexicon = Lexicon::builtin();
        lexicon.insert(lexeme("dataset", LexemeKind::Common, false));
        lexicon.insert(lexeme("GHz", LexemeKind::Unit, true));

        let canonicals: Vec<_> = lexicon
            .entries()
            .map(|lexeme| lexeme.canonical.as_str())
            .collect();
        assert_eq!(canonicals, ["AI", "CPU", "GPU", "API", "dataset", "GHz"]);
    }

    #[test]
    fn duplicate_batch_canonicals_keep_the_later_entry() {
        let mut lexicon = Lexicon::builtin();
        let mut first = lexeme("LLM", LexemeKind::Term, false);
        first.aliases = Vec::new();
        let mut second = lexeme("LLM", LexemeKind::Acronym, true);
        second.aliases = vec!["large language model".to_string()];
        second.requires_explanation = true;
        lexicon.overlay_workspace([first, second]);

        assert_eq!(
            lexicon
                .entries()
                .filter(|lexeme| lexeme.canonical == "LLM")
                .count(),
            1,
            "batch dedupes repeated canonicals"
        );
        let found = lexicon.lookup("LLM").expect("LLM resolves");
        assert_eq!(found.kind, LexemeKind::Acronym);
        assert!(found.case_sensitive);
        assert!(found.requires_explanation);
        assert!(lexicon.lookup("large language model").is_some());
        assert_eq!(lexicon.entries().count(), 5);
    }

    #[test]
    fn exact_key_shadows_folded_key_over_the_same_letters() {
        let mut lexicon = Lexicon::default();
        lexicon.insert(lexeme("cnn", LexemeKind::Term, false));
        lexicon.insert(lexeme("CNN", LexemeKind::Acronym, true));

        // The exact map answers first for the letters it owns.
        assert_eq!(lexicon.lookup("CNN").expect("exact wins").canonical, "CNN");
        assert_eq!(lexicon.lookup("cnn").expect("folded kept").canonical, "cnn");
        // Mixed case still folds onto the case-insensitive lexeme.
        assert_eq!(lexicon.lookup("Cnn").expect("folded kept").canonical, "cnn");
    }

    #[test]
    fn later_inserts_win_a_folded_key_conflict() {
        let mut lexicon = Lexicon::default();
        lexicon.insert(lexeme("Dataset", LexemeKind::Common, false));
        lexicon.insert(lexeme("dataset", LexemeKind::Term, false));

        let found = lexicon.lookup("DATASET").expect("folded hit");
        assert_eq!(found.canonical, "dataset", "last writer owns the key");
        // Both lexemes remain listed; only the disputed key is frozen.
        assert_eq!(lexicon.entries().count(), 2);
    }

    #[test]
    fn empty_surfaces_never_reach_the_scan_index() {
        let mut lexicon = Lexicon::default();
        let mut empty = lexeme("", LexemeKind::Term, false);
        empty.aliases = vec!["   ".to_string()];
        lexicon.insert(empty);
        lexicon.ensure_known_acronyms(["", "   "]);

        assert!(lexicon.lookup("").is_none());
        assert_eq!(lexicon.scan_keys().count(), 0);
    }

    #[test]
    fn scan_keys_mirror_the_frozen_index() {
        let mut lexicon = Lexicon::builtin();
        let mut entry = lexeme("dataset", LexemeKind::Common, false);
        entry.aliases = vec!["dataset".to_string(), "data set".to_string()];
        lexicon.insert(entry);

        let mut exact = std::collections::BTreeSet::new();
        let mut folded = std::collections::BTreeSet::new();
        for key in lexicon.scan_keys() {
            if key.folded {
                folded.insert((key.needle.to_string(), key.canonical.to_string()));
            } else {
                exact.insert((key.needle.to_string(), key.canonical.to_string()));
            }
        }
        // Alias equal to the canonical collapses to one folded key.
        assert_eq!(
            folded,
            [
                ("data set".to_string(), "dataset".to_string()),
                ("dataset".to_string(), "dataset".to_string())
            ]
            .into()
        );
        assert_eq!(
            exact,
            [
                ("AI".to_string(), "AI".to_string()),
                ("API".to_string(), "API".to_string()),
                ("CPU".to_string(), "CPU".to_string()),
                ("GPU".to_string(), "GPU".to_string()),
            ]
            .into()
        );
    }
}
