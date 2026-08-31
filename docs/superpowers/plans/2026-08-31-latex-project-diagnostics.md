# LaTeX Project Parsing and Compiler-Style Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recursively parse multi-file LaTeX projects with Tree-sitter, lint mapped Chinese/English logical text, and emit Ariadne compiler-style diagnostics from immutable source snapshots.

**Architecture:** Two isolated worktree lanes build independent foundations in parallel: the parser/rules lane owns logical documents and byte-span mapping, while the human-output lane owns Ariadne rendering against the already-public `SourceFile` and `Diagnostic` boundary. A serial integration writer applies both patches, wires the CLI and exit codes, and runs end-to-end tests; two fresh reviewers then inspect parser/rule correctness and output/portability independently.

**Tech Stack:** Rust 2024, tree-sitter 0.25, tree-sitter-latex 0.1, regex, serde/TOML, Clap, Ariadne 0.6, concolor or equivalent Ariadne-compatible stdout color detection, assert_cmd, tempfile

**Spec:** `docs/superpowers/specs/2026-08-31-latex-project-diagnostics-design.md`

## Global Constraints

- Support only Tree-sitter `latex_include`: `\\input`, `\\include`, `\\subfile`, and `\\subfileinclude`; do not implement `import_include`.
- `Span.start..Span.end` is a half-open UTF-8 byte range into the exact original `SourceFile.text`; preserve original CR/LF bytes.
- Logical text maps back to original LaTeX source; renderer snippets always show original LaTeX, never normalized logical text.
- `Diagnostic` remains presentation-free; JSON contains no ANSI or UI fields.
- Render from loaded `SourceFile` snapshots; output code must not re-read diagnostic paths.
- Existing `--format text` remains accepted as an alias for canonical `--format human`.
- Human/JSON reports use stdout; operational errors use stderr.
- Exit codes are `0` for no error-level diagnostic, `1` when any error-level diagnostic exists, and `2` for operational failure.
- No `help`, secondary labels, ASCII charset flag, terminal clipping, new lint rule families, or unrelated refactoring.
- Every behavior change follows red-green TDD; do not weaken a failing regression test to make implementation pass.

---

### Task 1: Tree-sitter Logical Project and Chinese Rules

**Parallel lane:** Wave 1, parser/rules worktree. This task runs concurrently with Task 2 and must not edit Task 2-owned files.

**Files:**
- Modify: `src/latex/span.rs`
- Modify: `src/latex/parser.rs`
- Modify: `src/latex/project.rs`
- Modify: `src/rules/mod.rs`
- Modify: `src/config/rules.rs`
- Modify: `src/config/defaults.rs`
- Modify: `src/text/sentence.rs`
- Test: `tests/latex_project.rs`
- Test: `tests/rules_logical_text.rs`
- Test: `tests/config_defaults.rs`
- Do not modify: `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/cli.rs`, or `src/output/**`

**Interfaces:**
- Consumes: existing `Span`, `SourceFile`, `TextSegment`, `LatexConfig`, `Document`, and `Rule::check` public concepts.
- Produces:
  - `Document { entry: PathBuf, root: PathBuf, sources: Vec<SourceFile>, blocks: Vec<TextBlock> }`, where `root` is the canonical parent directory of canonical `entry` (field names may remain public for current crate style).
  - `TextBlock { text: String, mappings: Vec<SourceMapping> }` where each `SourceMapping` maps a logical byte range to one original source span.
  - `Document::source_span(&TextBlock, Range<usize>) -> Option<Span>` (an equivalently named API is acceptable only if it has both mapping and immutable source-snapshot access and rules do not duplicate mapping logic).
  - `Document::source(&Path) -> Option<&SourceFile>` or equivalent deterministic snapshot lookup.
  - `Sentence` exposes an exact logical UTF-8 byte range whose start/end account for trimmed leading/trailing whitespace.
  - `parser::parse(path: PathBuf, config: &LatexConfig) -> Result<Document, ParseError>`.
  - `Style001Config { level, max_chars, max_english_words }`, with serde alias `max_words` for `max_english_words` and default `max_chars = 80` when omitted.

- [ ] **Step 1: Write recursive project regression tests**

Create `tests/latex_project.rs` using `tempfile::tempdir()` and literal files. The first tests must exercise the real public parser, not mocks:

```rust
#[test]
fn recursively_expands_latex_includes_in_reading_order() {
    // main.tex: "开头。\\input{chapters/one}结尾。"
    // chapters/one.tex: "第一章。\\include{nested}"
    // chapters/nested.tex: "嵌套正文。"
    // Assert block text in reading order contains 开头 → 第一章 → 嵌套正文 → 结尾.
    // Assert all three canonical SourceFile paths are retained exactly once.
}

#[test]
fn recognizes_all_latex_include_commands_and_extensionless_tex_paths() {
    // One fixture each for input/include/subfile/subfileinclude.
    // Each child contains a distinct literal marker; assert every marker appears once.
}

#[test]
fn include_cycle_is_a_contextual_parse_error() {
    // a.tex inputs b; b.tex inputs a.
    // Assert Err text names both files or an explicit include cycle.
}
```

Also add tests for relative-to-including-file resolution, missing include context, and duplicate completed-file suppression.

- [ ] **Step 2: Run project tests and verify RED**

Run:

```bash
cargo test --test latex_project -- --nocapture
```

Expected: compilation or assertions fail because `parser::parse` has no config parameter, no recursive includes, no logical blocks, and no cycle error.

- [ ] **Step 3: Implement source-backed logical document primitives**

In `src/latex/span.rs`, retain `SourceFile` and add focused mapping types. The canonical shape is:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapping {
    pub logical: std::ops::Range<usize>,
    pub source: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    pub text: String,
    pub mappings: Vec<SourceMapping>,
}

impl Document {
    pub fn source_span(
        &self,
        block: &TextBlock,
        logical: std::ops::Range<usize>,
    ) -> Option<Span> {
        // Find first and last overlapping mappings, require one source file,
        // map partial boundary offsets when logical/source fragment lengths match,
        // fetch that immutable SourceFile snapshot, and derive one-based
        // line/Unicode-scalar column from original text (never display width).
    }
}
```

Keep mapping logic in one reusable document/source-map method rather than in each rule. Byte offsets and source file are canonical; line/column are derived only with source text available.

- [ ] **Step 4: Implement Tree-sitter parsing and AST include discovery**

In `src/latex/parser.rs`, instantiate a real parser and configure:

```rust
let language = tree_sitter_latex::LANGUAGE.into();
parser.set_language(&language)?;
let tree = parser.parse(source.as_bytes(), None).ok_or(...)?;
```

Walk AST nodes in source order. Detect includes only by `node.kind() == "latex_include"`, then read the named `path` field and nested `path` node bytes. Do not use regex for command discovery.

Implement the spec's explicit node-kind policy. Retain structural title/text fields, caption text, generic formatting-command arguments, whitespace, and sentence punctuation. Skip complete subtrees for comments, math, citations, labels/references, command/environment definitions, class/package/bibliography/graphics/include resources, and configured ignored environments. For non-ignored generic environments, skip begin/end declarations and descend into body prose. Flush blocks at source-file and include boundaries.

- [ ] **Step 5: Implement recursive include path resolution and cycle handling**

In `src/latex/project.rs`, centralize path behavior:

```rust
pub fn resolve_include(including_file: &Path, requested: &Path) -> io::Result<PathBuf> {
    // parent(requesting file) + requested;
    // exact existing path first;
    // if requested.extension().is_none(), fallback to with_extension("tex");
    // canonicalize the selected path.
}
```

The recursive loader owns:

```rust
visiting: HashSet<PathBuf> // current recursion stack; hit => cycle error
loaded: HashSet<PathBuf>   // completed files; hit => skip duplicate linting
```

Errors must identify the including file and requested include.

- [ ] **Step 6: Run project tests and verify GREEN**

Run:

```bash
cargo test --test latex_project -- --nocapture
```

Expected: all recursive loading, command-family, path, duplicate, missing-file, and cycle tests pass.

- [ ] **Step 7: Write logical extraction and byte mapping regression tests**

Extend `tests/latex_project.rs` with real fixtures:

```rust
#[test]
fn extraction_retains_prose_and_excludes_non_prose_ast_nodes() {
    // Source includes chapter/section titles, caption, a formatting command,
    // %/block comments, inline/display/math environments, ignored lstlisting,
    // citation, label/reference, command definition, and package/class/
    // bibliography/graphics resource paths.
    // Assert title/caption/我们采用BERT模型。 and required whitespace/punctuation remain.
    // Assert every non-prose marker/path is absent from logical blocks.
}

#[test]
fn maps_utf8_child_match_to_original_latex_bytes() {
    // Child source contains "前缀\\textbf{中文问题}后缀。"
    // Locate 中文问题 in logical block, call source_span, and assert:
    // source.file == canonical child path;
    // &source_text[span.start..span.end] == "中文问题";
    // line/column are one-based and point into the original source.
}
```

Include a CRLF source fixture and assert byte slicing still selects the literal target without newline normalization. Add a sentence-segmentation regression with leading spaces/newlines and assert its exact logical byte range starts at the first non-whitespace byte and ends after the sentence terminator.

- [ ] **Step 8: Run mapping tests and verify RED, then implement extraction/mapping**

Run the exact new test names, confirm expected failures, then implement the smallest AST extraction and mapping builder that passes them:

```bash
cargo test --test latex_project maps_utf8_child_match_to_original_latex_bytes -- --exact --nocapture
cargo test --test latex_project extraction_retains_prose_and_excludes_non_prose_ast_nodes -- --exact --nocapture
```

Re-run all `latex_project` tests after implementation.

- [ ] **Step 9: Write Chinese-aware STYLE001 and mapped-rule tests**

Create `tests/rules_logical_text.rs` with parser + real `RuleEngine` tests. Use config overrides to turn unrelated rules off. Required literal cases:

```rust
#[test]
fn reports_chinese_sentence_over_max_chars_at_child_source_span() {
    // main inputs chapter; chapter has >80 CJK chars ending in 。
    // Assert one STYLE001, child path, exact sentence byte slice, warning severity.
}

#[test]
fn does_not_report_short_chinese_sentence() { /* below max_chars */ }

#[test]
fn english_uses_max_english_words_and_mixed_uses_effective_length() { /* literal wants */ }

#[test]
fn acronym_and_term_matches_point_to_exact_logical_source_ranges() {
    // Assert ACR001/TERM001 no longer point to an entire file.
}
```

- [ ] **Step 10: Run rule tests and verify RED**

Run:

```bash
cargo test --test rules_logical_text -- --nocapture
```

Expected: current raw-document rules cannot consume blocks, Chinese punctuation is missed, and spans cover the whole entry document.

- [ ] **Step 11: Wire all implemented rules to ordered logical blocks**

Update `src/rules/mod.rs`:

- ACR001: match each block and map each matched range.
- ACR002: count across blocks and retain the first occurrence span for each acronym.
- TERM001: match configured wrong terms per block and map each term range.
- STYLE001: extend `src/text/sentence.rs` so each sentence carries an exact logical UTF-8 byte range after trim offsets; call segmentation/language/statistics utilities per block; Chinese/Mixed compare to `max_chars`, English to `max_english_words`; map the offending logical sentence range.

Do not implement acronym-definition semantics or new rule families.

- [ ] **Step 12: Add STYLE001 config compatibility test and implementation**

In `tests/config_defaults.rs`, first add:

```rust
#[test]
fn style001_old_max_words_alias_keeps_default_chinese_limit() {
    let raw: RawPaperlintConfig = toml::from_str(r#"
[rules.STYLE001]
level = "warning"
max_words = 7
"#).unwrap();
    let config = PaperlintConfig::from_raw(raw, DefaultConfig::load());
    assert_eq!(config.rules.style001.max_chars, 80);
    assert_eq!(config.rules.style001.max_english_words, 7);
}
```

Verify RED, then add `#[serde(default = "...")] max_chars` and `#[serde(alias = "max_words")] max_english_words`, and update defaults.

- [ ] **Step 13: Verify Task 1 and commit**

Run:

```bash
cargo fmt --check
cargo test --test latex_project
cargo test --test rules_logical_text
cargo test --test config_defaults
cargo test --lib
```

Review `git diff --check`, confirm no Task 2-owned file changed, then commit:

```bash
git add src/latex src/rules/mod.rs src/config src/text/sentence.rs tests/latex_project.rs tests/rules_logical_text.rs tests/config_defaults.rs
git commit -m "fix(latex): parse recursive projects with source mapping"
```

Return the commit hash, changed files, exact test commands/results, and any interface concern. Do not push or merge.

---

### Task 2: Ariadne Human Renderer

**Parallel lane:** Wave 1, human-output worktree. This task runs concurrently with Task 1 and must not edit Task 1-owned files.

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `src/output/human.rs`
- Modify: `src/output/mod.rs`
- Test: `tests/output_human.rs`
- Do not modify: `src/latex/**`, `src/rules/**`, `src/config/**`, `src/main.rs`, or `src/cli.rs`

**Interfaces:**
- Consumes current public `SourceFile { path, text }`, `Diagnostic { rule, severity, message, span }`, and canonical byte spans.
- Produces:

```rust
pub enum ColorMode { Auto, Always, Never }

pub fn render(
    sources: &[SourceFile],
    root: &Path,
    diagnostics: &[Diagnostic],
    color: ColorMode,
) -> Result<String, HumanRenderError>;
```

An equivalent writer-based internal helper is encouraged, but this public adapter must be easy for Task 3 to call without disk I/O. `HumanRenderError` must preserve Ariadne I/O and UTF-8 conversion context and include `MissingSource(PathBuf)` plus invalid-byte-range context. The renderer prevalidates every diagnostic path/range before invoking Ariadne because Ariadne may otherwise skip a missing cached label without returning an error.

- [ ] **Step 1: Add failing deterministic renderer tests**

Create `tests/output_human.rs`. Build literal `SourceFile` and `Diagnostic` values directly; do not write/read files in renderer tests. Required first test:

```rust
#[test]
fn renders_original_chinese_source_with_rule_location_and_summary() {
    let text = "第一行。\n本文对该问题进行了详细分析。\n";
    // Compute byte offsets with literal find only in test setup.
    // Render ColorMode::Never.
    // Assert warning[STYLE001], slash-normalized relative file path,
    // original Chinese source line, a source marker/frame, and
    // "Found 1 problem: 0 errors, 1 warning".
    // Assert no "\x1b[".
}
```

Also add cases for an error+warning summary, `No problems found.`, and source-not-found returning an error rather than re-reading disk.

- [ ] **Step 2: Run renderer tests and verify RED**

Run:

```bash
cargo test --test output_human -- --nocapture
```

Expected: `output::human`, Ariadne dependency, `ColorMode`, and rendering API do not exist.

- [ ] **Step 3: Add focused dependencies and renderer skeleton**

Add Ariadne `0.6.0`. Enable only the color capability actually used. If Ariadne's `auto-color` feature cannot honor explicit `always/never` without a direct shared override, add only the matching `concolor 0.1` API/auto features; do not add a second terminal styling framework.

In `src/output/human.rs`:

- convert source IDs to relative `/`-normalized display strings;
- build an in-memory `ariadne::sources(...)` cache from `SourceFile.text.clone()`;
- use `Config::with_index_type(IndexType::Byte)`, Unicode charset, tab width `4`, and explicit color decision;
- use `Report::write_for_stdout`, never `FileCache`;
- preserve `warning[RULE]`/`error[RULE]` by using `ReportKind::Custom` with standard Yellow/Red and no `with_code`; Ariadne owns only source-frame layout;
- append one `\n`-normalized summary after all reports and strip/avoid `\r` in renderer-owned output.

Map `Level::Error` to custom red `error[RULE]` and `Level::Warning` to custom yellow `warning[RULE]`; `Level::Off` should not normally appear and must not be rendered as an error.

- [ ] **Step 4: Run deterministic tests and verify GREEN**

Run:

```bash
cargo test --test output_human -- --nocapture
```

Expected: deterministic no-color tests pass with source excerpt and correct counts.

- [ ] **Step 5: Add multibyte, tab, CRLF, and path tests**

Add literal tests that prove:

- a UTF-8 byte range selecting `BERT` after Chinese text marks the intended source;
- a tab before a label renders without panic and retains the target line;
- CRLF source byte offsets select the intended second-line text and rendered output contains no `\r`;
- a nested path under a canonical entry root renders as the exact expected relative `/` path without modifying the stored `PathBuf`;
- `ColorMode::Always` includes `\x1b[` and `ColorMode::Never` does not.

If global color override makes same-process tests racy, keep exact color-mode assertions in separate integration-test processes or serialize only those tests; do not weaken the contract.

- [ ] **Step 6: Run focused tests and self-review output boundaries**

Run:

```bash
cargo fmt --check
cargo test --test output_human
cargo test --test contracts
```

Inspect `rg -n "read_to_string|FileCache" src/output` and confirm the human renderer does not access disk. Inspect JSON code and confirm it has no dependency on Ariadne or color types.

- [ ] **Step 7: Commit Task 2**

Review `git diff --check`, confirm no Task 1-owned file changed, then commit:

```bash
git add Cargo.toml Cargo.lock src/output tests/output_human.rs
git commit -m "feat(output): render source diagnostics with ariadne"
```

Return the commit hash, changed files, exact test commands/results, and any color API concern. Do not push or merge.

---

### Task 3: Integrate Parser, Renderer, CLI, and Exit Contracts

**Lane:** Serial integration writer after Tasks 1 and 2 finish. Start from the plan branch and apply both Wave 1 patches/commits into one isolated integration worktree. This writer is the only writer for shared files.

**Files:**
- Apply: all Task 1 changes
- Apply: all Task 2 changes
- Modify: `src/main.rs`
- Modify: `src/cli.rs`
- Modify: `src/output/json.rs`
- Modify: `src/output/mod.rs` if needed for final exports/`text` compatibility
- Modify or remove: `src/output/text.rs` while preserving module/API compatibility as appropriate
- Modify: `src/error.rs`
- Modify: `tests/cli_smoke.rs`
- Create: `tests/cli_project.rs`
- Modify: `tests/contracts.rs` only if serialization error API or canonical fields require it; do not add presentation fields
- Modify: `README.md`
- Modify: `docs/rules.md`

**Interfaces:**
- Consumes Task 1 `Document.sources/blocks/root` and Task 2 `human::render`.
- Produces canonical CLI:

```rust
#[derive(ValueEnum)]
pub enum OutputFormat {
    #[value(alias = "text")]
    Human,
    Json,
}

#[derive(ValueEnum)]
pub enum ColorChoice { Auto, Always, Never }
```

- Produces a private main-layer result carrying both `Document` and diagnostics so rendering uses the same source snapshots.

- [ ] **Step 1: Apply both Wave 1 handoffs and run focused suites**

Apply the full Task 1 and Task 2 patches/commits. Resolve no semantic conflicts by dropping behavior. Run:

```bash
cargo test --test latex_project
cargo test --test rules_logical_text
cargo test --test output_human
```

Expected: focused lane tests pass before shared wiring.

- [ ] **Step 2: Write failing CLI project and format/color tests**

Create `tests/cli_project.rs` with a temporary `main.tex` that includes a child containing:

- one ignored/default-safe English context;
- one configured TERM001 warning or STYLE001 warning;
- one non-ignored acronym error in a separate test.

Required cases:

```rust
#[test]
fn human_main_reports_included_chinese_source_and_warning_exits_zero() { /* --color never */ }
#[test]
fn json_main_reports_child_file_byte_span_without_ansi() { /* parse JSON */ }
#[test]
fn human_and_legacy_text_formats_are_both_accepted() { /* compare semantic output */ }
#[test]
fn color_always_emits_ansi_and_never_does_not() { /* separate CLI processes */ }
#[test]
fn error_diagnostic_exits_one_and_missing_include_exits_two() { /* literal codes */ }
#[test]
fn multiple_findings_have_deterministic_human_and_json_order() { /* exact ordered keys */ }
```

Update `tests/cli_smoke.rs` so its exit expectation reflects error-level diagnostics, not any output.

- [ ] **Step 3: Run CLI tests and verify RED**

Run:

```bash
cargo test --test cli_project -- --nocapture
cargo test --test cli_smoke -- --nocapture
```

Expected: `main.rs` still calls old parse/render interfaces, format/color flags are missing, and warning exit behavior is wrong.

- [ ] **Step 4: Wire the main pipeline without renderer-owned policy**

Refactor private `run` to return the loaded document and diagnostics together:

```rust
struct LintResult {
    document: Document,
    diagnostics: Vec<Diagnostic>,
}
```

Pipeline:

```text
resolve entry → load config → parser::parse(entry, &config.latex)
→ RuleEngine::run(&document, &config)
→ choose renderer using the same document.sources snapshot
→ choose exit code from diagnostic severities
```

Sort diagnostics deterministically before either renderer by display/source path, `span.start`, `span.end`, rule ID, then message. Human output must render even for zero diagnostics (`No problems found.`). JSON output must serialize `[]` for zero diagnostics. Write final bytes with `stdout().lock().write_all(...)` and map write failure to an operational error; operational errors go to stderr and exit `2`.

- [ ] **Step 5: Add canonical format/color CLI and compatibility alias**

In `src/cli.rs`, add `--color auto|always|never` default `auto`, canonical `--format human|json`, and `text` as a Clap alias for `human`. Keep the public types simple and map CLI `ColorChoice` to output `ColorMode` in `main.rs`.

- [ ] **Step 6: Make JSON/render failures explicit**

Change JSON rendering from silent fallback:

```rust
pub fn render(diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(diagnostics)
}
```

Add contextual `PaperlintError` variants or main-layer handling for JSON/human output failures. Never emit `[]` after serialization failure.

- [ ] **Step 7: Run CLI tests and verify GREEN**

Run:

```bash
cargo test --test cli_project -- --nocapture
cargo test --test cli_smoke -- --nocapture
cargo test --test contracts
```

Expected: all format, color, source, deterministic ordering, JSON, summary, stdout handling, and exit-code cases pass.

- [ ] **Step 8: Update public docs to match implemented behavior**

Update `README.md` and `docs/rules.md`:

- `--format human|json`, with `text` compatibility note;
- `--color auto|always|never`;
- exit codes based on error-level diagnostics;
- `STYLE001.max_chars` and `max_english_words`, with legacy `max_words` note if useful;
- supported recursive include commands and explicit non-support for import command family;
- compiler-style human output example using real implemented message shape.

Do not document planned help labels or ASCII mode as implemented.

- [ ] **Step 9: Run integration quality gates and commit**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
git diff --check
```

Commit the integrated implementation:

```bash
git add Cargo.toml Cargo.lock src tests README.md docs/rules.md
git commit -m "fix: lint recursive latex projects with source diagnostics"
```

Return commit(s), changed files, exact gate output summary, and residual risks. Do not push or merge.

---

### Task 4: Parallel Review Wave and Accepted Fixes

**Wave 2:** Two fresh read-only reviewers run concurrently against the exact integrated Task 3 head. They must inspect actual source and diff, not rely on worker summaries.

**Reviewer A — parser/rules/source-map scope:**
- Tree-sitter AST usage and command-family scope;
- include ordering, relative paths, `.tex` fallback, duplicate/cycle behavior;
- exclusion/retention of correct LaTeX nodes;
- canonical UTF-8 byte spans, CRLF, multibyte and formatting gaps;
- Chinese/Mixed/English STYLE001 thresholds;
- cross-block and cross-file rule behavior;
- focused tests that can fail for realistic mutations.

**Reviewer B — output/CLI/portability scope:**
- no renderer disk rereads;
- Ariadne byte index, source cache, path and newline display behavior;
- ANSI auto/always/never and JSON isolation;
- human/text compatibility;
- warning/error/operational exit codes;
- Windows path, CRLF, tabs, CJK alignment and deterministic no-color tests;
- docs matching actual CLI behavior.

Each reviewer returns only evidence-backed P0/P1/P2 findings with file/line references and ends with `Merge verdict: BLOCK`, `OK`, or `OK with notes`.

- [ ] **Step 1: Run both fresh reviews concurrently**

Provide each reviewer the spec path, plan path, exact integrated diff/branch, and its distinct scope above. Reviewers do not edit or run subagents.

- [ ] **Step 2: Classify every finding against current HEAD**

For each finding, classify it as valid blocker, valid non-blocker, stale, invalid, out-of-scope, or speculative. Preserve evidence and do not implement optional scope expansion.

- [ ] **Step 3: Dispatch one fix writer for all accepted blockers**

If any P0/P1 or acceptance-contract gap is valid, one writer applies all accepted fixes in the integration worktree, adds/updates a failing regression test first, and re-runs only affected tests before the full gates. Do not run parallel fix writers on overlapping source.

- [ ] **Step 4: Run focused parallel re-review**

Both reviewers independently verify only their previously accepted findings and any new breakage in the fix diff. Residual optional P2 items are recorded, not looped indefinitely.

- [ ] **Step 5: Verify final integrated branch**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
git diff --check
```

Expected: all pass with zero warnings.

---

### Task 5: Real BioMed-QAgent Acceptance Test

**Files:** No source changes unless the real project exposes a covered regression; any necessary fix returns to TDD and Task 4 review.

- [ ] **Step 1: Build the exact final binary**

Run:

```bash
cargo build
```

Use `target/debug/paperlint` for all acceptance runs so Cargo output cannot obscure CLI stdout/stderr or exit codes.

- [ ] **Step 2: Run human output against the real entry**

Run:

```bash
set +e
target/debug/paperlint "$HOME/Data/coding/BioMed-QAgent/docs/latex/main.tex" --format human --color never > /tmp/paperlint-biomed-human.txt 2> /tmp/paperlint-biomed-human.err
status=$?
set -e
```

Assert:

- status is `1` only if at least one ACR001 error exists, otherwise `0`;
- stderr is empty;
- output includes at least one `chapters/*.tex` source location;
- output includes at least one `STYLE001` warning sourced from Chinese text;
- output includes source frames and a final count summary;
- output has no ANSI escape sequence.

- [ ] **Step 3: Run JSON output and inspect source contracts**

Run:

```bash
set +e
target/debug/paperlint "$HOME/Data/coding/BioMed-QAgent/docs/latex/main.tex" --format json > /tmp/paperlint-biomed.json 2> /tmp/paperlint-biomed-json.err
status=$?
set -e
python3 -m json.tool /tmp/paperlint-biomed.json >/dev/null
```

Use a short Python assertion script to prove:

- diagnostics are non-empty;
- at least one diagnostic has `rule == "STYLE001"` and a chapter file path;
- every `span.start <= span.end`, line/column are one-based, and paths resolve to loaded project files;
- no serialized string contains `\x1b[`;
- exit status follows presence/absence of error-severity diagnostics.

- [ ] **Step 4: Inspect final diff and report evidence**

Run:

```bash
git status --short --branch
git diff --stat main...HEAD
git log --oneline main..HEAD
```

Report changed files, gate commands, real-project diagnostic counts by rule/severity/file, any deferred limitations, and the fact that no push/merge was performed.
