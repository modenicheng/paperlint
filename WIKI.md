# Paperlint Wiki

Welcome to Paperlint - a linter for Chinese and English academic papers written in LaTeX.

## Table of Contents

- [Getting Started](#getting-started)
- [User Guide](#user-guide)
- [Rule Reference](#rule-reference)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [Architecture](#architecture)

---

## Getting Started

### Installation

```bash
# From source
git clone https://github.com/modenicheng/paperlint.git
cd paperlint
cargo build --release
cargo install --path .
```

### Quick Start

```bash
# Lint a single LaTeX file
paperlint main.tex

# With custom config
paperlint main.tex --config paperlint.toml

# JSON output for tooling integration
paperlint main.tex --format json
```

### Exit Codes

- `0` - No errors found
- `1` - Lint errors detected
- `2` - Configuration or parsing failure

---

## User Guide

### Basic Usage

Paperlint analyzes your LaTeX documents for common writing issues in academic papers. It supports:

- ✅ Chinese academic papers with English terminology
- ✅ English academic papers
- ✅ Mixed Chinese-English content
- ✅ Multi-file LaTeX projects with `\input` and `\include`

### What Paperlint Checks

**Acronym Management (ACR)**
- Acronyms defined before use
- Unnecessary acronyms (defined but never used)

**Terminology Consistency (TERM)**
- Consistent use of technical terms
- Terminology explanations for specialized terms

**Style Issues (STYLE)**
- Sentence length (both Chinese and English)
- Weak verb overuse (进行、开展、实现 etc.)

**Chinese Grammar Patterns (SYN, FUNC)**
- Function word density
- Passive/active voice balance
- Prepositional phrase stacking
- Connector word overuse

### Multi-file Projects

Paperlint automatically follows `\input{}` and `\include{}` commands:

```latex
% main.tex
\documentclass{article}
\begin{document}
\input{sections/introduction}
\input{sections/method}
\end{document}
```

```bash
paperlint main.tex  # Analyzes all included files
```

### Output Formats

**Text (default)**
```
error[ACR001]: acronym `LLM` used before definition
  --> sections/introduction.tex:17:8
```

**JSON (for tools)**
```bash
paperlint main.tex --format json > results.json
```

```json
{
  "rule": "ACR001",
  "severity": "error",
  "message": "acronym `LLM` used before definition",
  "span": {
    "file": "sections/introduction.tex",
    "line": 17,
    "column": 8
  }
}
```

---

## Rule Reference

### ACR001: Acronym Definition Required

**Level:** Error (default)

Acronyms must be defined with their full form on first use.

**Good:**
```latex
大语言模型（Large Language Model，LLM）是一种新型模型。
后续可以直接使用 LLM 进行推理。
```

**Bad:**
```latex
本文使用 LLM 完成任务。  % ❌ LLM not defined
```

**Supported Formats:**
1. 中文名（English Full Name，ABC）
2. 中文名（English Full Name, ABC）
3. 中文名（ABC）[requires config]
4. english full name (ABC)

**Configuration:**
```toml
[rules.ACR001]
level = "error"
min_length = 2
ignore = ["AI", "CPU", "GPU", "API"]
allow_chinese_without_english_full_name = false
```

---

### ACR002: Unnecessary Acronym

**Level:** Warning (default)

Warns about acronyms that are defined but never used again.

**Example:**
```latex
大语言模型（Large Language Model，LLM）非常强大。
% LLM never used again in document
```

**Configuration:**
```toml
[rules.ACR002]
level = "warning"
min_usages_after_definition = 1
```

---

### TERM001: Terminology Consistency

**Level:** Warning (default)

Enforces consistent use of technical terms throughout the document.

**Example:**
```toml
[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"大型语言模型" = "大语言模型"
"Github" = "GitHub"
"数据集（Dataset）" = "数据集（dataset）"
```

When Paperlint finds "大型语言模型", it suggests using "大语言模型" instead.

---

### TERM002: Term Definition Required

**Level:** Warning (default)

Specialized terms should be explained on first use.

**Good:**
```latex
载体是指用于承载和传输信息的介质。
```

**Configuration:**
```toml
[rules.TERM002]
level = "warning"
context_chars = 100

[[rules.TERM002.terms]]
term = "检索增强生成"
require_explanation = true

[[rules.TERM002.terms]]
term = "载体"
require_explanation = true
```

---

### STYLE001: Long Sentence

**Level:** Warning (default)

Warns about overly long sentences.

**Chinese (character-based):**
```toml
[rules.STYLE001]
level = "warning"
max_chars = 80  # For Chinese text
```

**English (word-based):**
```toml
[rules.STYLE001]
max_english_words = 45
```

Paperlint automatically detects language and applies appropriate limits.

---

### FUNC001: Function Word Density

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Checks for excessive use of function words (虚词) in Chinese sentences.

**Configuration:**
```toml
[rules.FUNC001]
level = "warning"
max_ratio = 0.20
min_sentence_tokens = 10
words = ["的", "了", "在", "进行", "通过", "由于"]
```

---

### FUNC002: Repeated Function Word Classes

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Detects repeated use of similar function words in close proximity.

**Example issues:**
```latex
由于……，因此……，从而……，进而……
```

**Configuration:**
```toml
[rules.FUNC002]
level = "warning"
max_same_class_in_window = 3
window_tokens = 12

[rules.FUNC002.classes]
connective = ["由于", "因此", "从而", "进而"]
```

---

### STYLE002: Weak Verb Overuse

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Detects overuse of vague verbs in Chinese academic writing.

**Common weak verbs:**
- 进行 (to carry out)
- 开展 (to conduct)
- 实现 (to achieve)
- 完成 (to complete)

**Configuration:**
```toml
[rules.STYLE002]
level = "warning"
max_occurrences_per_paragraph = 3
words = ["进行", "开展", "实现", "完成", "做出", "具有"]
```

---

### SYN001: Ba-construction Density

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Checks density of 把字句 (ba-constructions) in Chinese text.

**Configuration:**
```toml
[rules.SYN001]
level = "warning"
max_per_paragraph = 2
```

---

### SYN002: Passive Construction Density

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Monitors use of passive voice (被字句) in Chinese text.

**Configuration:**
```toml
[rules.SYN002]
level = "warning"
max_per_paragraph = 2
max_consecutive_sentences = 2
```

---

### SYN003: Prepositional Phrase Stacking

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Detects excessive prepositional phrases before the main clause.

**Example issue:**
```latex
基于……，针对……，通过……，在……条件下，本文……
```

**Configuration:**
```toml
[rules.SYN003]
level = "warning"
max_prepositional_phrases_before_main_clause = 3
words = ["在", "基于", "通过", "针对", "对于", "根据"]
```

---

### SYN004: Connective Word Stacking

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Warns about too many connective words in a single sentence.

**Configuration:**
```toml
[rules.SYN004]
level = "warning"
max_connectives_per_sentence = 3
words = ["由于", "因此", "从而", "进而", "同时", "此外", "但是", "然而"]
```

---

### SYN005: Long Modifier Chain

**Level:** Warning (default)

**Status:** Infrastructure ready, implementation pending

Detects overly long modifier chains in Chinese text.

**Configuration:**
```toml
[rules.SYN005]
level = "warning"
max_modifier_tokens = 10
require_dependency = false  # v0.1: heuristic-based
```

---

## Configuration

### Configuration File

Create `paperlint.toml` in your project root:

```toml
# NLP backend configuration
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "none"  # Future: "ltp"

# LaTeX parser configuration
[latex]
ignore_environments = [
    "equation",
    "equation*",
    "align",
    "align*",
    "lstlisting",
    "minted",
    "verbatim",
]

# Rule configuration
[rules.ACR001]
level = "error"
min_length = 2
ignore = ["AI", "CPU", "GPU", "API"]

[rules.ACR002]
level = "warning"

[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"Github" = "GitHub"
"大型语言模型" = "大语言模型"

[rules.STYLE001]
level = "warning"
max_chars = 80
max_english_words = 45

# Disable a rule
[rules.FUNC001]
level = "off"
```

### Severity Levels

- `error` - Causes non-zero exit code
- `warning` - Reported but doesn't fail
- `off` - Rule disabled

### Command-line Overrides

```bash
# Enable specific rules
paperlint main.tex --enable ACR001,TERM001

# Disable specific rules
paperlint main.tex --disable STYLE001,FUNC001
```

### Language-specific Settings

Paperlint automatically detects text language:

- **Chinese-dominant text**: Uses character-based metrics
- **English-dominant text**: Uses word-based metrics
- **Mixed text**: Uses context-appropriate metrics

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

### Quick Contribution Guide

1. **Report Issues**
   - Use GitHub Issues
   - Include minimal LaTeX example
   - Specify expected vs actual behavior

2. **Propose New Rules**
   - Describe the writing issue it addresses
   - Provide examples of good and bad usage
   - Suggest default configuration

3. **Submit Code**
   - Fork and create feature branch
   - Add tests for new functionality
   - Run `cargo test` and `cargo fmt`
   - Update documentation

### Development Setup

```bash
git clone https://github.com/modenicheng/paperlint.git
cd paperlint
cargo build
cargo test
```

---

## Architecture

### Pipeline Overview

```
LaTeX Files
    ↓
Project Resolver (follows \input/\include)
    ↓
LaTeX Parser (tree-sitter-latex)
    ↓
Logical Document (with source spans)
    ↓
Text Analyzer
    ├─ Language Detection
    ├─ Sentence Segmentation
    └─ Terminology Extraction
    ↓
NLP Analyzer
    ├─ Lexical Analysis (Jieba)
    │   ├─ Tokenization
    │   └─ POS Tagging
    └─ Syntax Analysis (future)
    ↓
Rule Engine
    ↓
Diagnostics
```

### Key Design Principles

1. **Source Span Preservation**
   - Every token knows its original file location
   - Enables accurate error reporting

2. **Pluggable NLP Backends**
   - `LexicalAnalyzer` trait allows different tokenizers
   - Future syntax analyzers via `SyntaxAnalyzer` trait

3. **Capability-based Rules**
   - Rules declare required NLP features
   - Gracefully skip when features unavailable

4. **Language Agnostic Core**
   - Chinese-first design
   - English compatibility maintained
   - No hardcoded language assumptions

### Module Structure

```
src/
├── cli.rs              # Command-line interface
├── config/             # Configuration system
│   ├── defaults.rs     # Default rule settings
│   └── rules.rs        # Rule configuration types
├── latex/              # LaTeX parsing
│   ├── parser.rs       # tree-sitter integration
│   ├── project.rs      # Multi-file resolution
│   └── span.rs         # Source position tracking
├── lint/               # Linting engine
│   ├── diagnostic.rs   # Error/warning representation
│   ├── engine.rs       # Rule execution
│   └── registry.rs     # Rule registration
├── nlp/                # NLP analysis
│   ├── analyzer.rs     # Analyzer traits
│   └── jieba_analyzer.rs  # Jieba implementation
├── rules/              # Rule implementations
│   └── mod.rs          # Current rules
├── text/               # Text processing
│   ├── chars.rs        # Character statistics
│   ├── language.rs     # Language detection
│   ├── sentence.rs     # Sentence segmentation
│   └── terminology.rs  # Acronym extraction
└── output/             # Output formatting
    ├── text.rs         # Human-readable output
    └── json.rs         # JSON output
```

### Adding a New Rule

See [CONTRIBUTING.md#implementing-rules](./CONTRIBUTING.md#implementing-rules) for step-by-step guide.

---

## FAQ

**Q: Does Paperlint modify my LaTeX files?**

A: No. Paperlint only reads and reports issues. You must fix them manually.

**Q: Can I use Paperlint with Overleaf?**

A: Yes, download your project and run Paperlint locally, or integrate via Git.

**Q: Does it work with Chinese-only papers?**

A: Yes! Paperlint is designed with Chinese academic writing as the primary use case.

**Q: What about mathematical formulas?**

A: Math environments (equation, align, etc.) are ignored by default.

**Q: Can I integrate with my editor?**

A: JSON output mode enables LSP/editor integration. See [Editor Integration](./EDITOR_INTEGRATION.md).

**Q: How accurate is the NLP analysis?**

A: v0.1 uses Jieba for Chinese tokenization/POS tagging, which is production-grade. Some rules use heuristics that may have false positives.

**Q: Can I add custom rules?**

A: Not yet via configuration, but you can fork and add rules in `src/rules/`. Plugin system planned for future versions.

---

## Resources

- [GitHub Repository](https://github.com/modenicheng/paperlint)
- [Issue Tracker](https://github.com/modenicheng/paperlint/issues)
- [Changelog](./CHANGELOG.md)
- [Architecture Documentation](./ARCHITECTURE.md)

---

## License

MIT License - see [LICENSE](./LICENSE) file for details.

---

## Acknowledgments

- [tree-sitter-latex](https://github.com/latex-lsp/tree-sitter-latex) for LaTeX parsing
- [jieba-rs](https://github.com/messense/jieba-rs) for Chinese NLP
- Inspired by markdownlint, ESLint, and Clippy
