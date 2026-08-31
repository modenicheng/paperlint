# Paperlint

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange)]()

A linter for Chinese and English academic papers written in LaTeX. Designed specifically for Chinese academic writing with English terminology.

[English](#english) | [中文](#中文)

---

## English

### Features

- ✅ **Chinese-first design** - Built for Chinese academic papers with English terms
- ✅ **Acronym management** - Ensures acronyms are defined before use
- ✅ **Terminology consistency** - Enforces consistent technical term usage
- ✅ **Style checking** - Detects long sentences, weak verbs, and writing patterns
- ✅ **Multi-file support** - Automatically follows `\input` and `\include`
- ✅ **Configurable rules** - Fine-tune every rule to your needs
- ✅ **Fast and local** - Pure Rust, no network calls, respects privacy

### Quick Start

```bash
# Clone and build
git clone https://github.com/modenicheng/paperlint.git
cd paperlint
cargo build --release
cargo install --path .

# Lint your paper
paperlint main.tex

# With custom configuration
paperlint main.tex --config paperlint.toml

# JSON output for tooling
paperlint main.tex --format json
```

### Example Output

```
error[ACR001]: acronym `LLM` used before definition
  --> sections/introduction.tex:17:8

warning[TERM001]: use `大语言模型` instead of `大型语言模型`
  --> sections/method.tex:42:15

warning[STYLE001]: sentence has 92 characters; max is 80
  --> sections/background.tex:28:1
```

### Supported Formats

**Chinese Academic Paper:**
```latex
大语言模型（Large Language Model，LLM）是一种新型模型。
后续我们使用 LLM 进行实验。
```

**English Paper:**
```latex
We use large language model (LLM) for inference.
Then LLM processes the input.
```

### Configuration

Create `paperlint.toml`:

```toml
[rules.ACR001]
level = "error"
ignore = ["AI", "CPU", "GPU", "API"]

[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"Github" = "GitHub"
"大型语言模型" = "大语言模型"

[rules.STYLE001]
level = "warning"
max_chars = 80          # For Chinese text
max_english_words = 45  # For English text
```

### Available Rules

| Rule ID | Description | Status |
|---------|-------------|--------|
| **ACR001** | Acronym must be defined before use | ✅ Implemented |
| **ACR002** | Unnecessary acronym (defined but unused) | ✅ Implemented |
| **TERM001** | Terminology consistency | ✅ Implemented |
| **TERM002** | Term definition required | 🚧 Infrastructure ready |
| **STYLE001** | Long sentence | ✅ Implemented |
| **STYLE002** | Weak verb overuse (进行、开展、实现) | 🚧 Infrastructure ready |
| **FUNC001** | Function word density (虚词密度) | 🚧 Infrastructure ready |
| **FUNC002** | Repeated function word classes | 🚧 Infrastructure ready |
| **SYN001** | Ba-construction density (把字句) | 🚧 Infrastructure ready |
| **SYN002** | Passive voice density (被字句) | 🚧 Infrastructure ready |
| **SYN003** | Prepositional phrase stacking | 🚧 Infrastructure ready |
| **SYN004** | Connective word stacking | 🚧 Infrastructure ready |
| **SYN005** | Long modifier chains | 🚧 Infrastructure ready |

🚧 = NLP infrastructure complete, rule logic pending implementation

### Documentation

- [📖 User Guide & Wiki](./WIKI.md)
- [🛠️ Contributing Guide](./CONTRIBUTING.md)
- [📊 Implementation Status](./IMPLEMENTATION_STATUS.md)
- [🏗️ Architecture Overview](./IMPLEMENTATION_STATUS.md#architecture)

### Use Cases

**1. Chinese Academic Papers**
```latex
% Checks: acronym definitions, Chinese term consistency,
% sentence length (character-based), weak verbs
本文研究大语言模型（Large Language Model，LLM）...
```

**2. Mathematical Modeling Contest Papers**
```latex
% Checks: terminology, style, acronyms
我们建立了基于 LSTM 的预测模型...
```

**3. Course Reports**
```latex
% Checks: all rules, configurable strictness
实验采用了深度学习方法进行数据分析...
```

### Architecture

```
LaTeX Files
    ↓
Project Resolver → LaTeX Parser (tree-sitter)
    ↓
Logical Document (with source spans)
    ↓
Text Analyzer (language detection, segmentation)
    ↓
NLP Analyzer (Jieba: tokenization, POS tagging)
    ↓
Rule Engine
    ↓
Diagnostics (text/JSON output)
```

### Technology Stack

- **Language:** Rust
- **LaTeX Parsing:** tree-sitter-latex
- **Chinese NLP:** jieba-rs
- **Architecture:** Pluggable analyzers, capability-based rules

### Contributing

Contributions welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md).

**Ways to contribute:**
- 🐛 Report bugs and false positives
- 💡 Suggest new rules
- 📝 Improve documentation
- 🔧 Implement pending rules
- 🌐 Add translations

### Roadmap

- [x] Core NLP infrastructure (v0.1)
- [x] Acronym and terminology rules
- [ ] Implement Chinese style rules (FUNC*, STYLE002)
- [ ] Implement Chinese syntax rules (SYN*)
- [ ] Syntax analysis backend (LTP integration)
- [ ] LSP server for editor integration
- [ ] CI/CD integration examples
- [ ] Plugin system for custom rules

### License

[MIT License](./LICENSE)

### Citation

If you use Paperlint in academic work, please cite:

```bibtex
@software{paperlint2024,
  title = {Paperlint: A Linter for Chinese Academic Papers},
  author = {Paperlint Contributors},
  year = {2024},
  url = {https://github.com/modenicheng/paperlint}
}
```

### Acknowledgments

- [tree-sitter-latex](https://github.com/latex-lsp/tree-sitter-latex) - LaTeX parsing
- [jieba-rs](https://github.com/messense/jieba-rs) - Chinese NLP
- Inspired by markdownlint, ESLint, and Clippy

---

## 中文

### 功能特性

- ✅ **中文优先设计** - 专为中英文混排的中文学术论文打造
- ✅ **缩写管理** - 确保缩写在使用前定义
- ✅ **术语一致性** - 强制统一技术术语表达
- ✅ **文风检查** - 检测长句、空泛动词等写作问题
- ✅ **多文件支持** - 自动处理 `\input` 和 `\include`
- ✅ **规则可配置** - 每条规则都可精细调整
- ✅ **快速本地化** - 纯 Rust 实现，无网络请求，保护隐私

### 快速开始

```bash
# 克隆并编译
git clone https://github.com/modenicheng/paperlint.git
cd paperlint
cargo build --release
cargo install --path .

# 检查论文
paperlint main.tex

# 使用自定义配置
paperlint main.tex --config paperlint.toml

# JSON 输出（用于工具集成）
paperlint main.tex --format json
```

### 输出示例

```
error[ACR001]: 缩写 `LLM` 在定义前使用
  --> sections/introduction.tex:17:8

warning[TERM001]: 建议使用 `大语言模型` 而不是 `大型语言模型`
  --> sections/method.tex:42:15

warning[STYLE001]: 该句包含 92 个字符，建议不超过 80
  --> sections/background.tex:28:1
```

### 支持的格式

**推荐格式（完整）：**
```latex
大语言模型（Large Language Model，LLM）是一种新型模型。
```

**其他支持格式：**
```latex
% 英文逗号
检索增强生成（Retrieval-Augmented Generation, RAG）

% 仅中文+缩写（需配置允许）
大语言模型（LLM）

% 纯英文
large language model (LLM)
```

### 配置示例

创建 `paperlint.toml`：

```toml
# NLP 后端配置
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "none"  # 未来支持 "ltp"

# LaTeX 解析配置
[latex]
ignore_environments = [
    "equation",
    "align",
    "lstlisting",
    "verbatim",
]

# 规则配置
[rules.ACR001]
level = "error"
min_length = 2
ignore = ["AI", "CPU", "GPU", "API"]

[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"Github" = "GitHub"
"大型语言模型" = "大语言模型"
"数据集（Dataset）" = "数据集（dataset）"

[rules.STYLE001]
level = "warning"
max_chars = 80              # 中文按字符数
max_english_words = 45      # 英文按单词数
```

### 规则列表

| 规则 ID | 说明 | 状态 |
|---------|------|------|
| **ACR001** | 缩写必须先定义后使用 | ✅ 已实现 |
| **ACR002** | 无必要缩写（定义后未使用） | ✅ 已实现 |
| **TERM001** | 术语统一性 | ✅ 已实现 |
| **TERM002** | 专业术语需要解释 | 🚧 基础设施就绪 |
| **STYLE001** | 长句检测 | ✅ 已实现 |
| **STYLE002** | 空泛动词滥用（进行、开展、实现） | 🚧 基础设施就绪 |
| **FUNC001** | 虚词密度过高 | 🚧 基础设施就绪 |
| **FUNC002** | 同类虚词连续出现 | 🚧 基础设施就绪 |
| **SYN001** | 把字句密度 | 🚧 基础设施就绪 |
| **SYN002** | 被动句密度 | 🚧 基础设施就绪 |
| **SYN003** | 介词结构堆叠 | 🚧 基础设施就绪 |
| **SYN004** | 连接词堆积 | 🚧 基础设施就绪 |
| **SYN005** | 超长修饰链 | 🚧 基础设施就绪 |

🚧 = NLP 基础设施已完成，规则逻辑待实现

### 使用场景

**1. 中文学术论文**
```latex
% 检查：缩写定义、中文术语一致性、句子长度（字符）、空泛动词
本文研究大语言模型（Large Language Model，LLM）...
```

**2. 数学建模竞赛论文**
```latex
% 检查：术语、文风、缩写
我们建立了基于 LSTM 的预测模型...
```

**3. 课程报告**
```latex
% 检查：所有规则，可配置严格程度
实验采用了深度学习方法进行数据分析...
```

### 技术架构

```
LaTeX 文件
    ↓
项目解析器 → LaTeX 解析器 (tree-sitter)
    ↓
逻辑文档（保留源位置）
    ↓
文本分析器（语言检测、句子切分）
    ↓
NLP 分析器（结巴分词：分词、词性标注）
    ↓
规则引擎
    ↓
诊断输出（文本/JSON）
```

### 技术栈

- **编程语言：** Rust
- **LaTeX 解析：** tree-sitter-latex
- **中文 NLP：** jieba-rs
- **架构：** 可插拔分析器，基于能力的规则系统

### 参与贡献

欢迎贡献！详见 [CONTRIBUTING.md](./CONTRIBUTING.md)。

**贡献方式：**
- 🐛 报告 bug 和误报
- 💡 建议新规则
- 📝 改进文档
- 🔧 实现待开发规则
- 🌐 添加翻译

### 开发路线

- [x] 核心 NLP 基础设施 (v0.1)
- [x] 缩写和术语规则
- [ ] 实现中文文风规则 (FUNC*, STYLE002)
- [ ] 实现中文句法规则 (SYN*)
- [ ] 句法分析后端（集成 LTP）
- [ ] LSP 服务器（编辑器集成）
- [ ] CI/CD 集成示例
- [ ] 自定义规则插件系统

### 许可证

[MIT 许可证](./LICENSE)

### 引用

如果在学术工作中使用 Paperlint，请引用：

```bibtex
@software{paperlint2024,
  title = {Paperlint: 中文学术论文检查工具},
  author = {Paperlint 贡献者},
  year = {2024},
  url = {https://github.com/modenicheng/paperlint}
}
```

### 致谢

- [tree-sitter-latex](https://github.com/latex-lsp/tree-sitter-latex) - LaTeX 解析
- [jieba-rs](https://github.com/messense/jieba-rs) - 中文分词
- 设计灵感来自 markdownlint、ESLint 和 Clippy

---

**Star ⭐ this repo if you find it useful!**

**如果觉得有用，请给个 Star ⭐！**
