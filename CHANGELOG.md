# Changelog

All notable changes to Paperlint will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Shared term lexicon (`[[lexicon.entries]]`) with a per-run `DocumentTermRegistry` and `LintContext`; all term-aware rules resolve surfaces through one lexicon instead of re-scanning (workspace overrides built-ins; `ACR001.ignore` stacks in)
- **CASE001**: canonical casing for known acronyms, proper nouns, and units (built-in core + workspace lexicon + document declarations); TERM001 defers case-only replacements; ACR001 stays silent for surfaces CASE001 owns
- **TERM002**: deterministic explanation-on-first-use for lexicon terms with `requires_explanation = true` (Chinese/English trigger patterns; later explanations never cancel a missing first-use one)
- **PUNC002**: missing space between CJK and Latin/number, with source-mapping-gap skipping and configurable ignore patterns (`第\d+章`, `图\d+`, `表\d+` defaults)
- ACR001 denoising: built-in cross-domain acronym core plus lexicon knowledge suppresses false "used before definition" reports (real-paper ACR001: 166 → 111)
- Lexicon occurrence scanning uses Aho-Corasick matchers compiled once per run (500-entry lexicon scan ~2× faster than per-key `match_indices`)
- Stdin input: `cat main.tex | paperlint`, omitted `INPUT`, and explicit `paperlint -` lint one in-memory LaTeX source with `<stdin>` spans; recursive includes remain available through file input
- `rust-toolchain.toml` (stable channel + rustfmt/clippy) — Rust 2024 edition requires 1.85+; contributors no longer hit mysterious toolchain errors
- Git hooks via lefthook: `pre-commit` (fmt + check), `commit-msg` (Conventional Commits validation via `scripts/commit-msg.sh`), `pre-push` (clippy -D warnings + tests)

### Changed

- Human diagnostics now use a compact three-line layout, crop source excerpts to 80 display columns around the finding, cap headers at 100 columns, and align markers for CJK text, emoji, tabs, CRLF, and multiline spans
- CI: dropped redundant `cargo build` (clippy/test already compile), added `--all-targets --all-features` to clippy/test, replaced three `actions/cache@v3` steps with `Swatinem/rust-cache@v2`, bumped `actions/checkout` to v4
- Docs consolidated: rule reference moved to `docs/rules.md`; removed WIKI/QUICKSTART/PROJECT_SUMMARY/COMPLETE_SUMMARY/IMPLEMENTATION_STATUS (overlapping content that would drift)
- CONTRIBUTING: Rust 1.85+ requirement, lefthook setup and workflow, commit types aligned with the validator (added `perf`, `build`, `ci`, `revert`)

### Previously (initial commit)

- Chinese NLP infrastructure with jieba-rs integration
- Text processing layer (language detection, sentence segmentation, character statistics)
- Terminology extraction supporting multiple Chinese academic paper formats
- Lexical analyzer interface with pluggable tokenizer support
- POS tagging support for Chinese text
- Rule configurations for 8 additional Chinese NLP rules
- Comprehensive documentation (WIKI.md, CONTRIBUTING.md, README.md)

### In Progress

- FUNC001: Function word density checking
- FUNC002: Repeated function word class detection
- STYLE002: Weak verb overuse detection
- SYN001-SYN005: Chinese syntax pattern rules

## [0.1.0] - 2026-08-31

### Added

- Initial release of Paperlint
- Core linting engine with configurable rules
- LaTeX project resolution with multi-file support (`\input`, `\include`)
- Source span tracking for accurate diagnostics
- **ACR001**: Acronym definition required before use
- **ACR002**: Unnecessary acronym detection
- **TERM001**: Terminology consistency enforcement
- **STYLE001**: Long sentence detection (Chinese and English)
- Configuration system with TOML support
- Command-line interface with text and JSON output
- Rule severity levels (error, warning, off)
- Boolean shorthand for rule configuration (`rule = false`)

### NLP Infrastructure

- Language detection (Chinese/English/Mixed)
- Sentence segmentation for Chinese and English
- Character-based statistics for Chinese text
- Word-based statistics for English text
- Acronym extraction with multiple format support:
  - 中文名（English Full Name，ABC）
  - 中文名（English Full Name, ABC）
  - 中文名（ABC）
  - english full name (ABC)
- Jieba tokenizer integration
- POS tagging for Chinese text
- Token-to-source position mapping

### Architecture

- Pluggable NLP backend system
- Capability-based rule system
- Clean separation: LaTeX parsing → Text analysis → NLP analysis → Rules
- Trait-based extensibility for analyzers

### Documentation

- Comprehensive user guide (WIKI.md)
- Contributing guidelines (CONTRIBUTING.md)
- Implementation status documentation
- Bilingual README (English/Chinese)
- MIT License

### Technical Details

- Written in Rust
- Uses tree-sitter-latex for parsing
- Uses jieba-rs for Chinese NLP
- 23 passing tests
- Zero warnings build

### Configuration

- Default configurations for all rules
- Configurable severity levels
- Customizable word lists for terminology rules
- Ignore lists for acronyms
- Language-specific thresholds (character/word counts)

### Known Limitations

- Sentence segmentation may have false positives with abbreviations/decimals
- English acronym extraction may capture extra words (v0.1 heuristic)
- No syntax analysis yet (dependency parsing planned for future)
- Only Jieba tokenizer backend available

## Development Milestones

### Phase 1: Foundation ✅ (v0.1)

- [x] Project structure
- [x] LaTeX parsing with tree-sitter
- [x] Multi-file project resolution
- [x] Source span tracking
- [x] Configuration system
- [x] Rule engine framework
- [x] CLI with text/JSON output

### Phase 2: NLP Infrastructure ✅ (v0.1)

- [x] Language detection
- [x] Sentence segmentation
- [x] Character/word statistics
- [x] Terminology extraction
- [x] Lexical analyzer interface
- [x] Jieba integration
- [x] POS tagging
- [x] Token position mapping

### Phase 3: Core Rules ✅ (v0.1)

- [x] ACR001: Acronym definition
- [x] ACR002: Unnecessary acronym
- [x] TERM001: Terminology consistency
- [x] STYLE001: Long sentences

### Phase 4: Chinese Style Rules 🚧 (Next)

- [ ] FUNC001: Function word density
- [ ] FUNC002: Repeated function words
- [ ] STYLE002: Weak verb overuse
- [ ] TERM002: Term definition required

### Phase 5: Chinese Syntax Rules 🚧 (Future)

- [ ] SYN001: Ba-construction density
- [ ] SYN002: Passive voice density
- [ ] SYN003: Prepositional phrase stacking
- [ ] SYN004: Connective word stacking
- [ ] SYN005: Long modifier chains

### Phase 6: Advanced NLP 📋 (Future)

- [ ] Syntax analyzer interface implementation
- [ ] LTP backend integration
- [ ] Dependency parsing
- [ ] Advanced syntax rules

### Phase 7: Tooling 📋 (Future)

- [ ] LSP server for editor integration
- [ ] CI/CD integration examples
- [ ] Pre-commit hooks
- [ ] GitHub Actions workflow

### Phase 8: Extensibility 📋 (Future)

- [ ] Plugin system for custom rules
- [ ] Custom analyzer backends
- [ ] Rule marketplace/registry
- [ ] Web-based rule tester

## Breaking Changes

None yet (initial release).

## Migration Guides

### From Pre-release to v0.1

This is the initial release, no migration needed.

## Contributors

See [CONTRIBUTORS.md](./CONTRIBUTORS.md) for the full list.

## Feedback and Issues

Report issues at: <https://github.com/modenicheng/paperlint/issues>

---

**Legend:**

- ✅ Completed
- 🚧 In Progress
- 📋 Planned
- ❌ Deprecated
