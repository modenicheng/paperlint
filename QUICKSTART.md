# Paperlint Quick Start Guide

Get started with Paperlint in 5 minutes!

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/modenicheng/paperlint.git
cd paperlint

# Build and install
cargo build --release
cargo install --path .

# Verify installation
paperlint --version
```

## Your First Lint

### 1. Create a Test LaTeX File

Create `test.tex`:

```latex
\documentclass{article}
\usepackage[UTF8]{ctex}

\begin{document}

\section{引言}

本文使用 LLM 进行实验。

大语言模型（Large Language Model，LLM）是一种新型模型，
它能够理解和生成自然语言文本，在各种自然语言处理任务中
都取得了显著的效果，包括但不限于文本分类、情感分析、
机器翻译、问答系统、对话生成等多种应用场景。

我们使用Github上的开源数据集。

\end{document}
```

### 2. Run Paperlint

```bash
paperlint test.tex
```

### 3. See the Results

You should see output like:

```
error[ACR001]: acronym `LLM` used before definition
  --> test.tex:8:9

warning[TERM001]: use `GitHub` instead of `Github`
  --> test.tex:14:5

warning[STYLE001]: sentence has 67 characters; max is 80
  --> test.tex:10:1
```

### 4. Fix the Issues

Update `test.tex`:

```latex
\documentclass{article}
\usepackage[UTF8]{ctex}

\begin{document}

\section{引言}

大语言模型（Large Language Model，LLM）是一种新型模型。
本文使用 LLM 进行实验。

LLM 能够理解和生成自然语言文本。
在各种自然语言处理任务中都取得了显著的效果。

我们使用 GitHub 上的开源数据集。

\end{document}
```

### 5. Run Again

```bash
paperlint test.tex
```

✅ No errors! Your paper passes all checks.

## Customizing Rules

### Create Configuration File

Create `paperlint.toml` in the same directory:

```toml
# Relax sentence length for Chinese
[rules.STYLE001]
level = "warning"
max_chars = 100

# Add custom terminology
[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"Github" = "GitHub"
"数据集（Dataset）" = "数据集（dataset）"
"大型语言模型" = "大语言模型"

# Ignore common acronyms
[rules.ACR001]
level = "error"
ignore = ["AI", "ML", "NLP", "CPU", "GPU"]
```

### Use the Configuration

```bash
paperlint test.tex --config paperlint.toml
```

## Multi-File Projects

### Project Structure

```
paper/
├── main.tex
├── sections/
│   ├── introduction.tex
│   ├── method.tex
│   └── conclusion.tex
└── paperlint.toml
```

### main.tex

```latex
\documentclass{article}
\usepackage[UTF8]{ctex}

\begin{document}

\input{sections/introduction}
\input{sections/method}
\input{sections/conclusion}

\end{document}
```

### Run Paperlint

```bash
cd paper
paperlint main.tex
```

Paperlint automatically checks all included files!

## Common Scenarios

### Scenario 1: Acronym Management

**Problem:**
```latex
我们使用 RAG 技术。
% ❌ ACR001: RAG not defined
```

**Solution:**
```latex
我们使用检索增强生成（Retrieval-Augmented Generation，RAG）技术。
后续实验中，RAG 显示出良好的效果。
% ✅ Defined before use
```

### Scenario 2: Terminology Consistency

**Problem:**
```latex
第一章使用"大型语言模型"。
第二章使用"大语言模型"。
% ❌ TERM001: Inconsistent terminology
```

**Solution:**

Add to `paperlint.toml`:
```toml
[rules.TERM001.replace]
"大型语言模型" = "大语言模型"
```

Then fix your paper to use "大语言模型" consistently.

### Scenario 3: Long Sentences

**Problem:**
```latex
本文提出了一种基于深度学习的方法，该方法首先对输入数据进行预处理，
然后通过多层神经网络提取特征，接着使用注意力机制进行特征选择，
最后通过全连接层输出预测结果，在多个数据集上进行了大量实验。
% ❌ STYLE001: Too long (92 characters)
```

**Solution:**
```latex
本文提出了一种基于深度学习的方法。
该方法首先对输入数据进行预处理，然后通过多层神经网络提取特征。
接着使用注意力机制进行特征选择，最后通过全连接层输出预测结果。
实验在多个数据集上验证了方法的有效性。
% ✅ Split into shorter sentences
```

### Scenario 4: Disable Specific Rules

**Temporarily disable a rule:**

```bash
paperlint main.tex --disable STYLE001
```

**Disable in configuration:**

```toml
[rules.STYLE001]
level = "off"
```

## JSON Output for Tools

### Generate JSON Report

```bash
paperlint main.tex --format json > report.json
```

### JSON Format

```json
[
  {
    "rule": "ACR001",
    "severity": "error",
    "message": "acronym `LLM` used before definition",
    "span": {
      "file": "test.tex",
      "start": 156,
      "end": 159,
      "line": 8,
      "column": 9
    }
  }
]
```

### Use in Scripts

```bash
# Check if there are any errors
paperlint main.tex --format json | jq 'map(select(.severity == "error")) | length'

# Extract all error messages
paperlint main.tex --format json | jq -r '.[] | select(.severity == "error") | .message'
```

## CI/CD Integration

### GitHub Actions Example

Create `.github/workflows/paperlint.yml`:

```yaml
name: Paperlint

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install Paperlint
        run: |
          git clone https://github.com/modenicheng/paperlint.git
          cd paperlint
          cargo install --path .
      
      - name: Lint Paper
        run: paperlint main.tex
```

## Tips and Tricks

### Ignore Math Environments

By default, math is ignored. To customize:

```toml
[latex]
ignore_environments = [
    "equation",
    "equation*",
    "align",
    "align*",
    "lstlisting",
    "minted",
    "verbatim",
    "algorithm",  # Add your own
]
```

### Check Multiple Files

```bash
# Check all .tex files in a directory
find . -name "*.tex" -exec paperlint {} \;

# Or use a loop
for file in *.tex; do
    echo "Checking $file"
    paperlint "$file"
done
```

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Run paperlint on staged .tex files

STAGED_TEX=$(git diff --cached --name-only --diff-filter=ACM | grep "\.tex$")

if [ -n "$STAGED_TEX" ]; then
    echo "Running paperlint on staged files..."
    for file in $STAGED_TEX; do
        paperlint "$file"
        if [ $? -ne 0 ]; then
            echo "❌ Paperlint found issues in $file"
            exit 1
        fi
    done
    echo "✅ All checks passed!"
fi
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Getting Help

### Command-line Help

```bash
paperlint --help
```

### Rule Documentation

See [WIKI.md](./WIKI.md) for detailed rule documentation.

### Report Issues

https://github.com/modenicheng/paperlint/issues

### Contribute

See [CONTRIBUTING.md](./CONTRIBUTING.md) for contribution guidelines.

## Next Steps

1. ✅ **Basic usage** - You've completed this guide!
2. 📖 **Read the documentation** - Check out [WIKI.md](./WIKI.md)
3. ⚙️ **Customize configuration** - Tailor rules to your needs
4. 🤝 **Contribute** - Help improve Paperlint!

## Common Configurations

### For Chinese Papers

```toml
[rules.ACR001]
level = "error"
allow_chinese_without_english_full_name = false

[rules.STYLE001]
max_chars = 80
```

### For Mathematical Modeling Contests

```toml
[rules.ACR001]
level = "warning"  # Relaxed

[rules.TERM001]
level = "error"    # Strict consistency

[rules.STYLE001]
max_chars = 100    # Longer sentences OK
```

### For Course Reports

```toml
[rules.ACR001]
level = "warning"

[rules.STYLE001]
max_chars = 90
```

## Troubleshooting

### Build Errors

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Jieba Loading Issues

If you see "failed to load dictionary":

```bash
# The dictionary is embedded, but check for:
# 1. Sufficient memory
# 2. File permissions
# 3. Try debug mode: RUST_LOG=debug paperlint main.tex
```

### Configuration Not Working

```bash
# Verify config file location
ls -la paperlint.toml

# Check config syntax
cat paperlint.toml

# Use explicit path
paperlint main.tex --config /full/path/to/paperlint.toml
```

---

**Happy linting! 🎉**

For more details, see the full documentation in [WIKI.md](./WIKI.md).
