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
cat main.tex | paperlint
paperlint - < main.tex
paperlint main.tex --config paperlint.toml
paperlint main.tex --format json
cat main.tex | paperlint --format json
paperlint main.tex --format human --color never
```

Output formats are `human` (default) and `json`; the legacy `text` value remains an alias for `human`. Human diagnostics use a compact three-line layout and crop long physical source lines around the highlighted span. Human output supports `--color auto|always|never`. Exit codes are severity-based: `0` means no error-level diagnostics (warnings may be present), `1` means at least one lint error, and `2` means a CLI, configuration, project, parse, render, or output failure.

With no `INPUT`, or with `INPUT` set to `-`, Paperlint reads one UTF-8 LaTeX source from standard input and reports its path as `<stdin>`. Stdin is intentionally single-file: if the source contains `\input`, `\include`, `\subfile`, or `\subfileinclude`, pass the project entry file path instead so relative includes have a directory.

For file input, Paperlint recursively loads `\input`, `\include`, `\subfile`, and `\subfileinclude` in reading order. The `\import` command family (`\import`, `\subimport`, `\inputfrom`, and `\includefrom`) is not supported. BibTeX/reference metadata—including citation keys, bibliography commands, and `thebibliography` entries—is excluded from linting.

### Example Output

```text
warning[STYLE001] chapters/introduction.tex:42:1: sentence has 112 effective characters; max is 80
  …cropped original LaTeX source around the problem…
                    ^^^^^^^^^^^^^^^^^^^^^^^^…

Found 1 problem: 0 errors, 1 warning
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
min_length = 2
ignore = ["AI", "CPU", "GPU", "API"]

[rules.ACR002]
level = "warning"
min_usages_after_definition = 1

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

paperlint main.tex                      # 检查论文工程
cat main.tex | paperlint                # 从标准输入检查单文件
paperlint - < main.tex                  # 显式使用标准输入
paperlint main.tex --config paperlint.toml
paperlint main.tex --format json        # 工具集成
cat main.tex | paperlint --format json
paperlint main.tex --format human --color never
```

输出格式为 `human`（默认）或 `json`；旧的 `text` 值仍作为 `human` 的兼容别名。人类可读诊断采用紧凑的三行排版，过长的物理源码行会围绕问题位置自动裁剪。颜色可通过 `--color auto|always|never` 控制。退出码按诊断级别确定：`0` 表示没有 error 级诊断（可以有 warning），`1` 表示至少一个 lint error，`2` 表示 CLI、配置、工程、解析、渲染或输出失败。

省略 `INPUT` 或将其设为 `-` 时，Paperlint 从标准输入读取一个 UTF-8 LaTeX 源文件，诊断路径显示为 `<stdin>`。stdin 刻意采用单文件模式：如果源码包含 `\input`、`\include`、`\subfile` 或 `\subfileinclude`，应传入工程入口文件路径，以便按其目录解析相对引用。

使用文件输入时，Paperlint 按阅读顺序递归加载 `\input`、`\include`、`\subfile` 和 `\subfileinclude`。暂不支持 `\import` 命令族（`\import`、`\subimport`、`\inputfrom`、`\includefrom`）。BibTeX/参考文献元数据（包括引用键、书目命令和 `thebibliography` 条目）不属于论文正文，不参与检查。

### 输出示例

```text
warning[STYLE001] chapters/introduction.tex:42:1: sentence has 112 effective characters; max is 80
  …围绕问题位置裁剪后的原始 LaTeX 源码…
                    ^^^^^^^^^^^^^^^^^^^^^^^^…

Found 1 problem: 0 errors, 1 warning
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
min_length = 2
ignore = ["AI", "CPU", "GPU", "API"]

[rules.ACR002]
level = "warning"
min_usages_after_definition = 1

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
