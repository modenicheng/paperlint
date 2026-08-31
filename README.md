# Paperlint

[![CI](https://github.com/modenicheng/paperlint/actions/workflows/ci.yml/badge.svg)](./.github/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-orange)]()

A linter for Chinese and English academic papers written in LaTeX. Chinese-first: designed for 中文正文 + 英文术语/缩写 + 公式 + 多文件 LaTeX 工程, with English compatibility.

[English](#english) · [中文](#中文)

---

## English

### Quick Start

```bash
git clone https://github.com/modenicheng/paperlint.git
cd paperlint
cargo install --path .

paperlint main.tex
paperlint main.tex --config paperlint.toml
paperlint main.tex --format json
```

Exit codes: `0` no error · `1` lint errors · `2` config/parse failure.

### Example Output

```text
error[ACR001]: 缩写 `LLM` 在定义前使用
  --> sections/introduction.tex:17:8

warning[TERM001]: 建议使用统一术语 `大语言模型`，而不是 `大型语言模型`
  --> sections/method.tex:42:15
```

### Configuration

```toml
# paperlint.toml
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "none"

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
max_chars = 80          # Chinese: effective chars
max_english_words = 45  # English: words
```

### Rules

Implemented in v0.1: **ACR001** (acronym defined before use), **ACR002** (unnecessary acronym), **TERM001** (terminology consistency), **STYLE001** (long sentences, language-aware).

Planned: TERM002, FUNC001/002 (function words), STYLE002 (weak verbs), SYN001–005 (把/被字句, prepositional & connective stacking, modifier chains), PUNC001/002, dependency-gated syntax rules.

Full rule reference, configuration keys, and examples: **[docs/rules.md](./docs/rules.md)**.

### Architecture

```text
LaTeX → ProjectResolver → Parser → Logical Document
      → Text Analyzer (language, sentences, terminology)
      → NLP Analyzer (jieba: tokens + POS, span-preserving)
      → Rule Engine → Diagnostics
```

Same input + same config = same diagnostics. Deterministic, mechanical, reproducible.

### Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). Requires Rust 1.85+ (enforced via `rust-toolchain.toml`); git hooks via [lefthook](https://lefthook.dev).

### License

[MIT](./LICENSE)

---

## 中文

### 快速开始

```bash
git clone https://github.com/modenicheng/paperlint.git
cd paperlint
cargo install --path .

paperlint main.tex                      # 检查论文
paperlint main.tex --config paperlint.toml
paperlint main.tex --format json        # 工具集成
```

退出码：`0` 无 error · `1` 存在 lint error · `2` 配置或解析失败。

### 输出示例

```text
error[ACR001]: 缩写 `LLM` 在定义前使用
  --> sections/introduction.tex:17:8

warning[TERM001]: 建议使用统一术语 `大语言模型`，而不是 `大型语言模型`
  --> sections/method.tex:42:15
```

### 配置

```toml
# paperlint.toml
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "none"

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
max_chars = 80          # 中文按有效字符数
max_english_words = 45  # 英文按单词数
```

### 规则

v0.1 已实现：**ACR001**（缩写先定义后使用）、**ACR002**（无必要缩写）、**TERM001**（术语统一）、**STYLE001**（长句，中英文分别计量）。

规划中：TERM002、FUNC001/002（虚词）、STYLE002（空泛动词）、SYN001–005（把/被字句、介词与连接词堆积、超长修饰链）、PUNC001/002、依赖句法树的规则。

完整规则参考、配置项与示例见 **[docs/rules.md](./docs/rules.md)**。

### 架构

```text
LaTeX → ProjectResolver → Parser → 逻辑文档
      → 文本分析（语言检测、句子切分、术语提取）
      → NLP 分析（jieba 分词 + POS，保留源位置映射）
      → 规则引擎 → 诊断输出
```

同样输入 + 同样配置 = 同样诊断结果。机械、可解释、可复现。

### 参与贡献

见 [CONTRIBUTING.md](./CONTRIBUTING.md)。需要 Rust 1.85+（由 `rust-toolchain.toml` 保证）；Git hooks 使用 [lefthook](https://lefthook.dev)。

### 许可证

[MIT](./LICENSE)
