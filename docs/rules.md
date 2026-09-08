# Rule Reference

All rules are configured under `[rules.<ID>]` in `paperlint.toml`. Severity levels: `error`, `warning`, `off` (a rule can also be disabled with `<ID> = false`). IDs are a stable API.

Status legend: ✅ implemented · 🚧 infrastructure ready (logic pending).

## Acronyms

### ACR001 — Acronym must be defined before use ✅

**Level:** `error` (default)

An acronym must appear with its full form before it is used. All of these definition formats are recognized:

```latex
大语言模型（Large Language Model，LLM）   % recommended: full-width comma
检索增强生成（Retrieval-Augmented Generation, RAG）  % ASCII comma also accepted
大语言模型（LLM）
large language model (LLM)                % English papers
```

A use before the definition point is reported as:

```text
error[ACR001]: 缩写 `LLM` 在定义前使用
 --> sections/introduction.tex:17:8
```

```toml
[rules.ACR001]
level = "error"
min_length = 2
ignore = ["AI", "CPU", "GPU", "API"]   # tune per project
```

### ACR002 — Unnecessary acronym ✅

**Level:** `warning` (default)

Reports defined acronyms that are used fewer than `min_usages_after_definition` times after their first definition. The definition occurrence itself and any use before it are not counted:

```text
warning[ACR002]:
缩写 `LLM` 定义后未再次使用，没有必要引入该缩写
```

```toml
[rules.ACR002]
level = "warning"
min_usages_after_definition = 1
```

## Terminology

### TERM001 — Terminology consistency ✅

**Level:** `warning` (default)

Fully user-configured canonical terminology. Nothing is hardcoded:

```toml
[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"大型语言模型" = "大语言模型"
"检索增强产生" = "检索增强生成"
"Github" = "GitHub"
```

```text
warning[TERM001]:
建议使用统一术语 `大语言模型`，而不是 `大型语言模型`
```

### TERM002 — Term needs explanation on first use ✅

**Level:** `warning` (default)

Deterministic check: terms declared in `[[lexicon.entries]]` with `requires_explanation = true` must be explained near their **first** occurrence (structured declarations count as explanations; later explanations never cancel a missing first-use one). High-confidence triggers: Chinese `X是/是指/指的是/即/定义为/表示`, `所谓X`, `称X为`; English `X refers to / is defined as / denotes / stands for / (also known as ...) / , i.e., / :`.

`TERM002` does not report automatically discovered unknown candidates. `TermCandidate` values are shared, evidence-only registry data; only configured lexicon entries with `requires_explanation = true` enter this rule.

```toml
[rules.TERM002]
level = "warning"
context_chars = 100

[[lexicon.entries]]
canonical = "检索增强生成"
kind = "term"
requires_explanation = true
```

```text
warning[TERM002] sections/method.tex:12:8: technical term `载体` first used without a nearby explanation
```

## Capitalization

### CASE001 — Canonical casing for known terms ✅

**Level:** `warning` (default)

Reports case variants of canonical forms owned by the shared lexicon (built-in acronyms, proper nouns, units) or by document acronym declarations. Unknown mixed-case words stay silent; fully lowercase variants of all-uppercase acronyms (e.g. `rag` vs `RAG`) stay silent. TERM001 defers case-only replacements (e.g. `Github`→`GitHub`) to this rule.

```text
warning[CASE001] chapters/method.tex:7:12: use `PyTorch` instead of `Pytorch`
```

ACR001 also stays silent for uppercase surfaces whose case variants CASE001 owns (e.g. `PYTORCH`), so one wrong surface is never reported by both rules.

## Style

### STYLE001 — Long sentences ✅

**Level:** `warning` (default)

Chinese is measured by effective character count (CJK chars + latin words + numeric tokens); English by word count. Language is detected per sentence: more CJK than Latin characters ⇒ Chinese.

```toml
[rules.STYLE001]
level = "warning"
max_chars = 80
max_english_words = 45
```

The legacy `max_words` key remains accepted as an alias for `max_english_words`; `max_chars` defaults to `80` when omitted.

```text
warning[STYLE001]: sentence has 112 effective characters; max is 80
```

### STYLE002 — Weak verb overuse 🚧

**Level:** `warning` (default)

Counts vague verbs per paragraph (`进行 / 开展 / 实现 / 完成 / 做出 / 具有 / 得到`). Only warns; never rewrites:

```toml
[rules.STYLE002]
level = "warning"
max_occurrences_per_paragraph = 3
words = ["进行", "开展", "实现"]
```

```text
warning[STYLE002]:
“进行”在本段重复出现 4 次，可检查是否存在空泛动词
```

## Function words (Chinese)

Both rules run on tokens + POS from the lexical analyzer (jieba). They never mark a sentence as *wrong* — warnings only.

### FUNC001 — Function-word density 🚧

**Level:** `warning` (default)

Ratio of configured function words in a sentence:

```toml
[rules.FUNC001]
level = "warning"
max_ratio = 0.20
min_sentence_tokens = 10
words = ["的", "了", "在", "进行"]   # word list must stay configurable
```

```text
warning[FUNC001]:
该句虚词占比 31%，可能存在冗余表达
```

### FUNC002 — Same function-word class in a row 🚧

**Level:** `warning` (default)

Detects pile-ups like `由于……因此……从而……进而……` within a token window, based on configurable word classes:

```toml
[rules.FUNC002]
level = "warning"
max_same_class_in_window = 3
window_tokens = 12

[rules.FUNC002.classes]
connective = ["由于", "因此", "从而", "进而"]
```

## Syntax patterns (token/POS heuristics)

These need only segmentation + POS + local patterns — no dependency tree. Density-based: e.g. four 把-constructions in one paragraph is a finding, one is not.

### SYN001 — 把-construction density 🚧

```toml
[rules.SYN001]
level = "warning"
mode = "density"
max_per_paragraph = 2
```

```text
warning[SYN001]:
本段出现 4 个把字结构，建议检查是否存在过多处置式表达
```

Detection (v0.1, no dependency parsing): token `把` + surrounding POS context.

### SYN002 — 被字句 / passive density 🚧

Counts passives per sentence/paragraph and consecutive passive sentences. Word-sense collisions (e.g. 被子) are excluded via the following-token pattern:

```toml
[rules.SYN002]
level = "warning"
max_per_paragraph = 2
max_consecutive_sentences = 2
```

Future English support: `be + past participle`.

### SYN003 — Prepositional phrase stacking 🚧

Sentence-initial stacks like `基于……，针对……，通过……，在……条件下，本文……`:

```toml
[rules.SYN003]
level = "warning"
max_prepositional_phrases_before_main_clause = 3
words = ["在", "基于", "通过", "针对", "对于", "根据"]
```

### SYN004 — Connective stacking 🚧

Too many connectives (`由于 / 因此 / 从而 / 进而 / 同时 / 此外 / 但是 / 然而`) in one sentence:

```toml
[rules.SYN004]
level = "warning"
max_connectives_per_sentence = 3
```

### SYN005 — Long modifier chains 🚧

v0.1 heuristic on runs of consecutive nouns/adjectives/particles (误报高于真正的句法分析):

```toml
[rules.SYN005]
level = "warning"
max_modifier_tokens = 10
require_dependency = false
```

## Rules requiring dependency parsing (planned)

The following are **not** implemented as regex approximations. They are reserved for a future dependency backend; current configs must keep `[nlp] syntax = "none"`.

- 主语缺失 / 谓语残缺 / 宾语残缺
- 多层定语依附关系 / 悬垂修饰
- 复杂把字句 / 复杂被动句合理性
- 句法嵌套深度
- 主谓搭配异常 / 动宾搭配异常
- 指代对象判断

## NLP backend configuration

```toml
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "none"
```

Current accepted values are exactly `tokenizer = "jieba"`, `pos = "jieba"`, and `syntax = "none"`; unsupported values fail config loading. v0.1 uses jieba-rs (CWS + POS), fully native Rust. Sentence and token units are built once in `LintContext` as logical UTF-8 byte ranges; the resulting `DocumentAnalysis` feeds the shared `DocumentTermRegistry`, including term-candidate evidence. Logical ranges are mapped back to LaTeX source spans only when a consumer needs a physical location. Workspace lexicon canonical and alias surfaces without whitespace are injected into jieba as technical nouns.

## Punctuation rules (planned)

### PUNC001 — Chinese/English punctuation consistency

Detects `,`/`:`/`()` in Chinese context, with exceptions for `Large Language Model (LLM)`, `f(x)`, `printf(...)`.

### PUNC002 — Spacing between CJK and Latin/numbers ✅

`使用LLM进行推理` → suggests `使用 LLM 进行推理`, with ignore patterns:

```toml
[rules.PUNC002]
ignore_patterns = ["第\\d+章", "图\\d+", "表\\d+"]
```

## Shared lexicon

All term-aware rules (ACR001 denoising, CASE001, TERM002) resolve surfaces through one shared lexicon built per run. Workspace entries override built-ins with the same canonical form; `ACR001.ignore` acronyms stack in as known acronyms.

```toml
[[lexicon.entries]]
canonical = "GWAS"
kind = "acronym"          # common | term | acronym | proper_noun | unit | symbol
aliases = []
# case_sensitive = true    # defaults by kind
requires_explanation = false
```

Built-in entries are a conservative cross-domain core (`PDF`, `DNA`, `HTTP`, `JSON`, `GitHub`, `PyTorch`, `TensorFlow`, `LaTeX`, `Hz`, `kHz`, …). No domain terms ship built in; declare them per project.

The same run-scoped registry also retains deterministic `TermCandidate` evidence for unknown surfaces. Known lexicon entries and exact document-declared Chinese, English, and acronym surfaces are suppressed from that candidate list. Candidates have a surface, first logical location, sorted evidence, bounded score, and kind hint, but they are not diagnostics and are not serialized into CLI JSON output.

## LaTeX projects and configuration

Paperlint recursively expands `\input`, `\include`, `\subfile`, and `\subfileinclude` in reading order. Include paths are resolved relative to the including file, with `.tex` fallback for extensionless paths. The `\import` command family (`\import`, `\subimport`, `\inputfrom`, and `\includefrom`) is not supported.

```toml
[latex]
ignore_environments = [
    "equation", "equation*", "align", "align*", "gather",
    "verbatim", "lstlisting", "minted",
]
```

## Full example

```toml
# paperlint.toml
[nlp]
tokenizer = "jieba"
pos = "jieba"
syntax = "none"

[rules.ACR001]
level = "error"
min_length = 2
ignore = ["AI", "CPU", "GPU"]

[rules.ACR002]
level = "warning"
min_usages_after_definition = 1

[rules.TERM001]
level = "warning"

[rules.TERM001.replace]
"大型语言模型" = "大语言模型"
"Github" = "GitHub"

[rules.TERM002]
level = "warning"
context_chars = 100

[[lexicon.entries]]
canonical = "检索增强生成"
kind = "term"
requires_explanation = true

[rules.STYLE001]
level = "warning"
max_chars = 80
max_english_words = 45

[rules.STYLE002]
level = "warning"
max_occurrences_per_paragraph = 3
words = ["进行", "开展", "实现"]

[rules.FUNC001]
level = "warning"
max_ratio = 0.20
min_sentence_tokens = 10

[rules.SYN001]
level = "warning"
max_per_paragraph = 2

[rules.SYN002]
level = "warning"
max_per_paragraph = 2
max_consecutive_sentences = 2

[rules.SYN003]
level = "warning"
max_prepositional_phrases_before_main_clause = 3

[rules.SYN004]
level = "warning"
max_connectives_per_sentence = 3

[rules.SYN005]
level = "warning"
max_modifier_tokens = 10
require_dependency = false

[latex]
ignore_environments = [
    "equation", "equation*", "align", "align*",
    "gather", "verbatim", "lstlisting", "minted",
]
```

## CLI

```bash
paperlint main.tex                                  # human output, full project
cat main.tex | paperlint                            # human output, stdin single file
paperlint - < main.tex                              # explicit stdin
paperlint main.tex --config paperlint.toml
paperlint main.tex --format human --color never
cat main.tex | paperlint --format json              # stdin JSON for tooling
paperlint main.tex --format json                    # project JSON for tooling
```

Omitting `INPUT` or passing `-` reads one UTF-8 LaTeX source from stdin and uses `<stdin>` in diagnostic spans. Stdin is single-file input and rejects LaTeX include commands; pass an entry file path for recursive project expansion.

Canonical output formats are `human` and `json`; `text` remains accepted as a compatibility alias for `human`. Human diagnostics use a compact three-line layout. Source snippets are capped at 80 display columns and cropped around the diagnostic span with `…`; a trailing `…` on the marker means the span continues beyond the visible excerpt or onto another line. Human color control is `--color auto|always|never`; JSON never contains ANSI styling.

Exit codes: `0` no error-level diagnostics (warnings may be present) · `1` one or more lint errors · `2` CLI/configuration/project/parse/render/output failure.

Priority order: built-in defaults < `paperlint.toml` < CLI overrides.

BibTeX and reference metadata are not paper prose and are never linted. Paperlint excludes citation keys, `\\addbibresource`, `\\bibliography`, `\\bibliographystyle`, `\\printbibliography`, and the complete `thebibliography` environment (including `\\bibitem` entries).
