# Changelog

All notable changes to Paperlint will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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

## [0.1.0] - 2024-12-XX

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

Report issues at: https://github.com/modenicheng/paperlint/issues

---

**Legend:**
- ✅ Completed
- 🚧 In Progress
- 📋 Planned
- ❌ Deprecated
