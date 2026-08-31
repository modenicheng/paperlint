# Contributing to Paperlint

Thank you for your interest in contributing to Paperlint! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
- [Implementing Rules](#implementing-rules)
- [Git Hooks (lefthook)](#git-hooks-lefthook)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)

---

## Code of Conduct

Be respectful and constructive in all interactions. We aim to build a welcoming community for academic writing tools.

---

## Getting Started

### Prerequisites

- Rust 1.85 or later (required by edition 2024; installed automatically via `rust-toolchain.toml`)
- Git
- Basic understanding of LaTeX and academic writing conventions

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/modenicheng/paperlint.git
   cd paperlint
   ```

2. **Install Git Hooks (lefthook)**
   ```bash
   lefthook install
   ```

   Hooks keep commits fast (fmt + `cargo check` on commit; clippy + tests on push) and enforce Conventional Commits. See [Git hooks](#git-hooks-lefthook) below.

3. **Build**
   ```bash
   cargo build
   ```

4. **Run Tests**
   ```bash
   cargo test --all-targets --all-features
   ```

---

## Project Structure

```
paperlint/
├── src/
│   ├── cli.rs              # Command-line interface
│   ├── config/             # Configuration system
│   ├── latex/              # LaTeX parsing
│   ├── lint/               # Linting engine
│   ├── nlp/                # NLP analysis layer
│   ├── rules/              # Rule implementations
│   ├── text/               # Text processing utilities
│   └── output/             # Output formatting
├── tests/                  # Integration tests
├── docs/
│   └── rules.md            # Rule reference
├── CONTRIBUTING.md         # This file
└── Cargo.toml
```

### Key Modules

**`src/latex/`** - LaTeX document parsing and project resolution
- `parser.rs` - tree-sitter-latex integration
- `project.rs` - Multi-file document resolution
- `span.rs` - Source position tracking

**`src/text/`** - Language-agnostic text processing
- `language.rs` - Language detection (Chinese/English/Mixed)
- `sentence.rs` - Sentence boundary detection
- `chars.rs` - Character and word statistics
- `terminology.rs` - Acronym and terminology extraction

**`src/nlp/`** - Natural language processing
- `analyzer.rs` - Tokenization and POS tagging traits
- `jieba_analyzer.rs` - Chinese NLP using Jieba

**`src/rules/`** - Rule implementations
- Each rule should be its own module
- Rules implement the `Rule` trait

**`src/lint/`** - Linting engine
- `engine.rs` - Orchestrates rule execution
- `registry.rs` - Rule registration
- `diagnostic.rs` - Error/warning representation

---

## How to Contribute

### 1. Report Issues

**Good bug reports include:**
- Minimal LaTeX example that triggers the issue
- Expected behavior
- Actual behavior
- Paperlint version
- Operating system

**Example:**
```markdown
**Bug:** ACR001 false positive on bibliography

**Example:**
\begin{document}
The API is useful.
\bibliographystyle{plain}
\bibliography{refs}  % Contains "API" expansion
\end{document}

**Expected:** No error (definition in bibliography)
**Actual:** error[ACR001]: acronym `API` used before definition

**Version:** 0.1.0
**OS:** Ubuntu 22.04
```

### 2. Suggest Features

**Good feature requests include:**
- Description of the writing problem it addresses
- Examples of good vs bad usage
- Suggested rule ID and default severity
- References (style guides, academic conventions)

**Example:**
```markdown
**Feature:** Check for "很" overuse in Chinese academic writing

**Problem:** Overuse of "很" (very) weakens academic tone

**Bad:** 这个方法很好，效果很明显，速度很快。
**Good:** 这个方法效果显著，运行速度较快。

**Suggested Rule:** STYLE003
**Default Level:** Warning
**Configuration:**
- max_occurrences_per_paragraph
- alternative_suggestions

**References:**
- 《科技论文写作规范》
```

### 3. Improve Documentation

- Fix typos and unclear explanations
- Add examples to rule documentation
- Translate documentation (especially Chinese ↔ English)
- Add FAQ entries

---

## Implementing Rules

### Step 1: Define Rule ID

Add to `src/rule_id.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleId {
    // ... existing rules
    MyNewRule,  // Add here
}

impl RuleId {
    pub const ALL: [Self; N] = [
        // ... existing rules
        Self::MyNewRule,  // Add here
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            // ... existing rules
            Self::MyNewRule => "MYNEW001",
        }
    }
}

impl FromStr for RuleId {
    // Add case in from_str
}
```

### Step 2: Define Configuration

Add to `src/config/rules.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MyNewRuleConfig {
    pub level: Level,
    pub threshold: usize,
    #[serde(default)]
    pub words: Vec<String>,
}

// Add to RulesConfig
#[derive(Debug, Clone, PartialEq)]
pub struct RulesConfig {
    // ... existing
    pub mynewrule: MyNewRuleConfig,
}

// Add to RawRulesConfig
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct RawRulesConfig {
    // ... existing
    pub mynewrule: Option<RuleSetting<MyNewRuleConfig>>,
}

// Implement HasLevel trait
impl HasLevel for MyNewRuleConfig {
    fn set_level(&mut self, level: Level) {
        self.level = level;
    }
}

// Add to PaperlintConfig::from_raw
impl PaperlintConfig {
    pub fn from_raw(raw: RawPaperlintConfig, defaults: Self) -> Self {
        let mut config = defaults;
        // ... existing
        if let Some(rule) = raw.rules.mynewrule {
            config.rules.mynewrule = resolve(rule, config.rules.mynewrule.clone());
        }
        config
    }

    fn set_rule_level(mut self, rule: &str, level: Level) -> Self {
        match rule.parse::<RuleId>() {
            // ... existing
            Ok(RuleId::MyNewRule) => self.rules.mynewrule.level = level,
            Err(_) => {}
        }
        self
    }
}
```

### Step 3: Add Default Configuration

Add to `src/config/defaults.rs`:

```rust
impl DefaultConfig {
    pub fn load() -> PaperlintConfig {
        PaperlintConfig {
            // ... existing
            rules: RulesConfig {
                // ... existing
                mynewrule: MyNewRuleConfig {
                    level: Level::Warning,
                    threshold: 3,
                    words: vec!["word1".into(), "word2".into()],
                },
            },
            // ... rest
        }
    }
}
```

### Step 4: Export Configuration

Add to `src/config/mod.rs`:

```rust
pub use rules::{
    // ... existing
    MyNewRuleConfig,
    // ... rest
};
```

### Step 5: Implement the Rule

Create `src/rules/mynewrule.rs`:

```rust
use crate::{
    config::PaperlintConfig,
    latex::{parser::Document, span::Span},
    lint::diagnostic::Diagnostic,
    rule_id::RuleId,
    nlp::JiebaAnalyzer,
    text::segment_sentences,
};

pub struct MyNewRule;

impl super::Rule for MyNewRule {
    fn id(&self) -> RuleId {
        RuleId::MyNewRule
    }

    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic> {
        let rule_config = &config.rules.mynewrule;
        
        // Check if rule is enabled
        if !rule_config.level.is_enabled() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        
        // Create base span for sentence segmentation
        let base_span = Span {
            file: document.path.clone(),
            start: 0,
            end: document.text.len(),
            line: 1,
            column: 1,
        };
        
        // Segment into sentences
        let sentences = segment_sentences(&document.text, &base_span);
        
        // Option 1: Simple pattern matching
        for sentence in sentences {
            let count = rule_config.words.iter()
                .filter(|word| sentence.text.contains(word.as_str()))
                .count();
            
            if count > rule_config.threshold {
                diagnostics.push(Diagnostic {
                    rule: self.id(),
                    severity: rule_config.level,
                    message: format!(
                        "Found {} occurrences of target words (threshold: {})",
                        count, rule_config.threshold
                    ),
                    span: sentence.span.clone(),
                });
            }
        }
        
        // Option 2: Use NLP analysis
        // let analyzer = JiebaAnalyzer::new();
        // for sentence in sentences {
        //     let analysis = analyzer.analyze(&sentence.text, &sentence.span);
        //     
        //     // Check POS tags, token sequences, etc.
        //     for token in analysis.tokens {
        //         // Your logic here
        //     }
        // }
        
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_rule_detects_issue() {
        // Create test document
        let document = Document {
            path: PathBuf::from("test.tex"),
            text: "这是测试文本。".to_string(),
        };
        
        // Create test config
        let mut config = crate::config::DefaultConfig::load();
        config.rules.mynewrule.words = vec!["测试".into()];
        config.rules.mynewrule.threshold = 0;
        
        // Run rule
        let rule = MyNewRule;
        let diagnostics = rule.check(&document, &config);
        
        // Verify
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, RuleId::MyNewRule);
    }
}
```

### Step 6: Register the Rule

Add to `src/rules/mod.rs`:

```rust
mod mynewrule;

use crate::{
    config::PaperlintConfig,
    latex::parser::Document,
    lint::diagnostic::Diagnostic,
    rule_id::RuleId,
};

pub trait Rule {
    fn id(&self) -> RuleId;
    fn check(&self, document: &Document, config: &PaperlintConfig) -> Vec<Diagnostic>;
}

pub use mynewrule::MyNewRule;
```

Add to `src/lint/registry.rs`:

```rust
use crate::rules::*;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // ... existing
        Box::new(MyNewRule),
    ]
}
```

### Step 7: Add Tests

Create tests in `tests/mynewrule.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_mynewrule_detects_issue() {
    let dir = TempDir::new().unwrap();
    let tex_file = dir.path().join("test.tex");
    
    fs::write(
        &tex_file,
        r#"\documentclass{article}
\begin{document}
Test content with issue.
\end{document}"#,
    )
    .unwrap();

    Command::cargo_bin("paperlint")
        .unwrap()
        .arg(tex_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("MYNEW001"));
}

#[test]
fn test_mynewrule_respects_config() {
    let dir = TempDir::new().unwrap();
    let tex_file = dir.path().join("test.tex");
    let config_file = dir.path().join("paperlint.toml");
    
    fs::write(&tex_file, "Test content").unwrap();
    fs::write(
        &config_file,
        r#"
[rules.mynewrule]
level = "off"
"#,
    )
    .unwrap();

    Command::cargo_bin("paperlint")
        .unwrap()
        .arg(&tex_file)
        .arg("--config")
        .arg(&config_file)
        .assert()
        .success();
}
```

### Step 8: Update Documentation

Add rule documentation to `docs/rules.md`:

```markdown
### MYNEW001: My New Rule

**Level:** Warning (default)

**Status:** Implemented

Description of what the rule checks and why it matters.

**Good:**
\`\`\`latex
Example of correct usage
\`\`\`

**Bad:**
\`\`\`latex
Example of incorrect usage
\`\`\`

**Configuration:**
\`\`\`toml
[rules.mynewrule]
level = "warning"
threshold = 3
words = ["word1", "word2"]
\`\`\`
```

### Step 9: Update Tests

Update `tests/contracts.rs`:

```rust
assert_eq!(
    ids,
    [
        "ACR001", "ACR002", "TERM001", "STYLE001", 
        "FUNC001", "FUNC002", "STYLE002",
        "SYN001", "SYN002", "SYN003", "SYN004", "SYN005",
        "MYNEW001",  // Add here
    ]
);
```

---

## Git Hooks (lefthook)

Quality gates run locally so you find problems before CI does. We use [lefthook](https://lefthook.dev) — a single CLI binary, no Node/Python runtime required.

### Install

```bash
# install the binary (any one of these)
brew install lefthook                          # macOS
cargo install lefthook                         # via cargo
# or grab a release: https://github.com/evilmartians/lefthook/releases

lefthook install                               # wire hooks into .git/hooks
```

### What runs when

| Hook | Commands | Purpose |
|------|----------|---------|
| `pre-commit` | `cargo fmt --all -- --check` · `cargo check --all-targets --all-features` | fast feedback; `check` skips code generation |
| `commit-msg` | `sh scripts/commit-msg.sh` | validates Conventional Commits |
| `pre-push` | `cargo clippy --all-targets --all-features -- -D warnings` · `cargo test --all-targets --all-features` | heavier gates before sharing |

Commits stay fast because the expensive clippy/test compile is deferred to `pre-push`. Skip hooks only for throwaway pushes: `git push --no-verify`.

### Valid commit messages

```text
feat(parser): support nested input
fix(nlp): avoid empty-token panic
docs: update rule documentation
```

Types: `feat fix docs style refactor perf test build ci chore revert`; optional scope in `(...)`; `!` marks breaking changes. `Merge ...`/`Revert ...` headers generated by git are exempt.

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test text::

# Run specific test
cargo test test_language_detection
```

### Integration Tests

```bash
# Run integration tests only
cargo test --test '*'

# Run specific integration test file
cargo test --test mynewrule
```

### Test Guidelines

1. **Unit tests** for each module in the same file
2. **Integration tests** for end-to-end rule behavior
3. **Test both positive and negative cases**
4. **Test configuration overrides**
5. **Test multi-file LaTeX projects**

### Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_issue() {
        // Arrange
        let input = "...";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_false_positives() {
        // Test that correct usage doesn't trigger warnings
    }

    #[test]
    fn test_respects_configuration() {
        // Test that config options are honored
    }

    #[test]
    fn test_edge_cases() {
        // Test boundary conditions, empty input, etc.
    }
}
```

---

## Code Style

### Rust Style

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy

# Check for lints
cargo clippy -- -D warnings
```

### Naming Conventions

- **Files:** `snake_case.rs`
- **Modules:** `snake_case`
- **Types:** `PascalCase`
- **Functions:** `snake_case`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Rule IDs:** `CATEGORY001` (e.g., `ACR001`, `TERM001`)

### Code Organization

1. **Imports** - standard library, external crates, local modules
2. **Types** - structs, enums
3. **Trait implementations**
4. **Functions** - public, then private
5. **Tests** - at the bottom with `#[cfg(test)]`

### Documentation

```rust
/// Brief one-line summary.
///
/// More detailed explanation if needed. Explain what the function does,
/// not how it does it (that's what code is for).
///
/// # Examples
///
/// ```
/// let result = function(input);
/// assert_eq!(result, expected);
/// ```
///
/// # Errors
///
/// Describe error conditions if function returns Result.
///
/// # Panics
///
/// Describe panic conditions if any.
pub fn function(param: Type) -> ReturnType {
    // Implementation
}
```

---

## Submitting Changes

### Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/my-new-rule
   ```

2. **Make Changes**
   - Write code
   - Add tests
   - Update documentation

3. **Test Locally**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets --all-features
   ```

   Or just `git push` — the lefthook `pre-push` hook runs clippy and tests for you.

4. **Commit**
   ```bash
   git add .
   git commit -m "feat: add MYNEW001 rule for checking X"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/my-new-rule
   ```
   Then create PR on GitHub.

### Commit Message Format

Use conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `style` - Code style (formatting, no logic change)
- `refactor` - Code restructuring
- `perf` - Performance improvement
- `test` - Adding tests
- `build` - Build system or dependencies
- `ci` - CI configuration
- `chore` - Maintenance
- `revert` - Revert a previous commit

Commit messages are validated by the `commit-msg` hook (see below).

**Examples:**
```
feat(rules): add FUNC001 for function word density

Implements checking for excessive function words in Chinese text.
Configurable threshold and word list.

Closes #42
```

```
fix(parser): handle nested \input commands correctly

Previously would fail on recursive includes. Now properly tracks
include depth and prevents infinite loops.

Fixes #123
```

### PR Checklist

- [ ] Code follows project style (cargo fmt, cargo clippy)
- [ ] All tests pass (cargo test --all-targets --all-features)
- [ ] New tests added for new functionality
- [ ] Documentation updated (docs/rules.md, code comments)
- [ ] CHANGELOG.md updated
- [ ] Commit messages follow conventional format
- [ ] PR description explains what and why

### PR Review Process

1. Maintainer reviews code
2. Automated tests run via CI
3. Feedback addressed
4. Approval and merge

---

## Development Tips

### Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run -- main.tex

# Run specific test with output
cargo test test_name -- --nocapture
```

### Testing Against Real Papers

```bash
# Build and test on your own papers
cargo build --release
./target/release/paperlint /path/to/your/paper.tex
```

### NLP Analysis

When implementing rules that need NLP:

```rust
use crate::nlp::{JiebaAnalyzer, LexicalAnalyzer};

let analyzer = JiebaAnalyzer::new();
let analysis = analyzer.analyze(&sentence.text, &sentence.span);

// Access tokens
for token in analysis.tokens {
    println!("Token: {}, POS: {:?}", token.text, token.pos);
}
```

### Common Patterns

**Pattern 1: Word/phrase matching**
```rust
let count = config.words.iter()
    .filter(|word| text.contains(word.as_str()))
    .count();
```

**Pattern 2: Regex matching**
```rust
use regex::Regex;

let pattern = Regex::new(r"pattern").unwrap();
for cap in pattern.captures_iter(&text) {
    // Process matches
}
```

**Pattern 3: Token sequence matching**
```rust
for window in analysis.tokens.windows(2) {
    if matches!(window[0].pos, PosTag::Preposition) 
        && matches!(window[1].pos, PosTag::Noun) {
        // Found preposition + noun
    }
}
```

---

## Getting Help

- **GitHub Issues:** Ask questions, report bugs
- **Discussions:** General questions and ideas
- **Email:** cheng20070225@outlook.com

---

## Recognition

Contributors will be acknowledged in:
- CONTRIBUTORS.md
- Release notes
- Project README

Thank you for contributing to Paperlint! 🎉
