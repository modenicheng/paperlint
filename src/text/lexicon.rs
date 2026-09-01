//! Shared term lexicon for semantic lint rules.
//!
//! The lexicon merges three layers, later layers overriding earlier ones per
//! canonical form: built-in entries, workspace config entries
//! (`[[lexicon.entries]]`), and acronyms stacked from `ACR001.ignore` so the
//! legacy ignore list keeps working. Rules resolve surfaces through
//! [`Lexicon::lookup`] instead of re-deriving term knowledge on their own.

use crate::config::{LexiconEntryConfig, PaperlintConfig};
use serde::Deserialize;
use std::collections::HashMap;

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
    /// Materialize a workspace lexeme from its config entry.
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

/// A lookup table from term surfaces to their canonical lexemes.
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    lexemes: Vec<Lexeme>,
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
    /// stacked as known acronyms when not already present.
    pub fn from_config(config: &PaperlintConfig) -> Self {
        let mut lexicon = Self::builtin();
        lexicon.overlay_workspace(config.lexicon.entries.iter().map(Lexeme::from_entry));
        for acronym in &config.rules.acr001.ignore {
            lexicon.ensure_known_acronym(acronym);
        }
        lexicon
    }

    /// Overlay workspace entries; entries with a canonical form already
    /// present replace the earlier lexeme entirely.
    pub(crate) fn overlay_workspace(&mut self, entries: impl IntoIterator<Item = Lexeme>) {
        for entry in entries {
            self.insert(entry);
        }
    }

    /// Insert one lexeme, replacing any existing lexeme with the same
    /// canonical form.
    pub(crate) fn insert(&mut self, lexeme: Lexeme) {
        self.lexemes
            .retain(|existing| existing.canonical != lexeme.canonical);
        self.lexemes.push(lexeme);
        self.rebuild_index();
    }

    /// Stack an `ACR001.ignore` acronym as a known acronym unless the surface
    /// already resolves to some lexeme.
    pub(crate) fn ensure_known_acronym(&mut self, acronym: &str) {
        if self.lookup(acronym).is_none() {
            self.insert(Lexeme {
                canonical: acronym.to_string(),
                aliases: Vec::new(),
                kind: LexemeKind::Acronym,
                source: LexemeSource::AcronymIgnore,
                case_sensitive: LexemeKind::Acronym.default_case_sensitive(),
                requires_explanation: false,
            });
        }
    }

    /// Resolve a surface to its lexeme, trying an exact match first and then
    /// an ASCII case-folded match.
    pub fn lookup(&self, surface: &str) -> Option<&Lexeme> {
        let canonical = self
            .exact
            .get(surface)
            .or_else(|| self.folded.get(&surface.to_ascii_lowercase()))?;
        self.lexemes
            .iter()
            .find(|lexeme| &lexeme.canonical == canonical)
    }

    /// Iterate lexemes in insertion order (built-ins first).
    pub fn entries(&self) -> impl Iterator<Item = &Lexeme> {
        self.lexemes.iter()
    }

    fn rebuild_index(&mut self) {
        self.exact.clear();
        self.folded.clear();
        for lexeme in &self.lexemes {
            for surface in std::iter::once(&lexeme.canonical).chain(&lexeme.aliases) {
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
        lexicon.ensure_known_acronym("XYZ");
        lexicon.ensure_known_acronym("AI");

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
}
