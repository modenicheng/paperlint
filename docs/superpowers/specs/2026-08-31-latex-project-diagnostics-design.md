# LaTeX Project Parsing and Compiler-Style Diagnostics Design

**Date:** 2026-08-31  
**Status:** Approved  
**Scope:** Tree-sitter-based recursive LaTeX loading, logical-text source mapping, Chinese-aware `STYLE001`, and Ariadne human diagnostics

## Problem

Running Paperlint on `~/Data/coding/BioMed-QAgent/docs/latex/main.tex` currently returns exit code `0` with no diagnostics even though six chapter files contain the paper body. Running chapters directly produces mostly English acronym findings and misses a synthetic Chinese sentence longer than 80 CJK characters.

The failures have two confirmed causes:

1. `ProjectResolver` only canonicalizes the entry path and `parser::parse` only calls `read_to_string`; the existing Tree-sitter dependencies are unused and `\input`/`\include` are never expanded.
2. `STYLE001` only matches ASCII `. ! ?` and counts whitespace-separated words; it does not use the existing Chinese sentence segmentation, language detection, or effective-character counting code.

The current text renderer also discards source information already present in each `Diagnostic`. The same source database required for correct multi-file spans can support compiler-style human diagnostics without coupling presentation to the lint engine.

## Goals

1. Parse LaTeX with `tree-sitter-latex` and recursively expand the `latex_include` command family in reading order.
2. Extract logical natural-language text rather than linting raw LaTeX commands, comments, math, or ignored environments.
3. Preserve an immutable snapshot of every loaded source and map logical matches back to canonical UTF-8 byte ranges in the original source.
4. Make `STYLE001` detect Chinese, mixed, and English long sentences with language-specific thresholds.
5. Render human diagnostics with Ariadne, including severity, rule ID, source location, source excerpt, underline, and a summary.
6. Preserve machine-readable JSON as presentation-free data.
7. Provide deterministic, cross-platform behavior for non-colored output and documented exit codes.

## Non-Goals

This change does not implement:

- `import_include` commands such as `\import`, `\subimport`, `\inputfrom`, or `\includefrom`;
- bibliography, class, package, or graphics inclusion as paper-body recursion;
- new Chinese FUNC/SYN rule implementations;
- a rewrite of ACR001/ACR002 semantics beyond making them consume logical text and report precise spans;
- secondary labels, fixes, or `Diagnostic.help` fields;
- ASCII drawing mode, themes, icons, TrueColor, terminal-width clipping, or source wrapping;
- LSP, SARIF, or GitHub Actions renderers;
- source newline normalization.

## Core Invariants

1. `Span.start` and `Span.end` are half-open UTF-8 byte offsets into the exact `SourceFile.text` snapshot identified by `Span.file`.
2. Original LF, CRLF, or CR newlines are preserved. Tree-sitter, rules, JSON, and human rendering use the same in-memory source snapshot.
3. `Span.line` and `Span.column` are derived, one-based compatibility fields. They are not used as the canonical range or for terminal marker alignment.
4. Logical text may omit LaTeX syntax, but every reported range maps back to original LaTeX source bytes.
5. `Diagnostic` remains presentation-free. ANSI styles, source frames, path normalization, and summaries exist only in `src/output/`.
6. The human renderer receives already-loaded `SourceFile` values and never re-reads diagnostic files from disk.
7. Human reports go to stdout. Operational/configuration/parse errors go to stderr. JSON goes to stdout and never contains ANSI escapes.

## Logical Document Model

The project is represented as an ordered logical document:

```text
Document
├── entry: PathBuf
├── root: PathBuf                (canonical parent directory of entry)
├── sources: source snapshot collection
└── blocks: Vec<TextBlock>       (paper reading order)
    ├── text: String             (logical natural-language text)
    └── mappings                 (logical byte range → original source Span)
```

`SourceFile` stores a canonical file path and its exact text. The source collection contains each canonical path once and offers deterministic iteration and lookup.

A `TextBlock` never crosses a source-file boundary. It combines adjacent natural-language fragments from a paragraph-like region while preserving mapping entries. Omitting formatting syntax may create a source gap between adjacent logical fragments; mapping a diagnostic across that gap produces a valid original-source range covering the intervening LaTeX syntax.

Rules consume `Document.blocks` in order. This keeps future cross-chapter state possible while preventing sentence segmentation from joining the end of one file to the start of another.

## Tree-sitter Parsing

Each source file is parsed with:

```rust
parser.set_language(&tree_sitter_latex::LANGUAGE.into())
```

The AST is the authority for command kinds and arguments. Regex must not be used to discover include commands.

### Recursive include commands

This release supports the grammar's `latex_include` node:

- `\input`
- `\include`
- `\subfile`
- `\subfileinclude`

For each node:

1. Read the AST `path` field.
2. Resolve it relative to the including file's directory.
3. Try the path exactly as written.
4. If it has no extension and the exact path does not exist, try the same path with `.tex`.
5. Canonicalize the resolved file before source lookup and cycle detection.
6. Flush the current text block and recursively expand the child at the command's position.

A recursion-stack hit is a parse/project error that names the cycle. Re-including a file that has already been completely loaded is skipped so the same physical source is linted once. A missing include is an operational parse error with the including file and requested path.

### Natural-language extraction

The parser emits source-backed logical fragments in AST order. It excludes:

- line and block comments;
- displayed, inline, delimiter, and math environments;
- verbatim/listing/minted and other configured `latex.ignore_environments`;
- package/class/bibliography/graphics/include paths;
- citations, labels, references, command/environment definitions, and command names.

Natural-language extraction follows an explicit node policy:

- retain prose by descending through structural nodes (`part`, `chapter`, `section`, `subsection`, `subsubsection`, `paragraph`, `subparagraph`, `enum_item`) and prose-bearing fields such as their title/text arguments;
- retain `caption` text and natural-language arguments of generic formatting commands, while skipping each command-name child;
- retain source whitespace and sentence punctuation between accepted prose leaves so English word boundaries and Chinese/English sentence boundaries survive extraction;
- skip the complete subtree for comments, math, include/resource commands, citations, labels/references, command/environment definitions, and configured ignored environments;
- for a non-ignored generic environment, skip `begin`/`end` declarations but descend through its body.

Natural-language arguments of formatting and structural commands remain visible. For example:

```latex
我们采用\textbf{BERT}模型。
```

produces logical text equivalent to:

```text
我们采用BERT模型。
```

but a diagnostic for `BERT` maps to the original four bytes' character range inside `\textbf{...}`. Section/chapter titles and captions are retained because they are prose.

Recoverable Tree-sitter `ERROR` nodes do not by themselves abort linting. Failure to configure the grammar, cancellation/no tree, unreadable files, missing includes, and include cycles are reported as operational parse errors.

## Source Mapping and Spans

Mapping is byte-based end to end:

- Tree-sitter node ranges are byte ranges.
- Logical block ranges are UTF-8 byte ranges in `TextBlock.text`.
- Mapping entries point from logical byte subranges to original source byte subranges.
- `Document::source_span(block, logical_range)` (or an equivalent API with access to both the block mappings and source snapshots) converts rule matches before constructing a `Diagnostic`; a `TextBlock`-only method must not guess derived line/column data without source text.

For a match spanning multiple mapped fragments in the same block, the primary source span starts at the first overlapping source byte and ends at the last overlapping source byte. This may include omitted formatting syntax, which is preferable to pointing at normalized text that does not exist in the displayed source.

Line and column are derived from the source snapshot at span construction time. `line` is one-based. `column` is a one-based Unicode scalar position for JSON/backward compatibility; Ariadne derives terminal layout from byte spans and source text. Sentence segmentation must expose exact logical byte ranges after trimming leading/trailing whitespace; source-like sentence spans with unadjusted trim offsets are not sufficient.

## Rule Integration

All implemented rules inspect logical blocks, never `Document`'s raw entry file.

- **ACR001:** regex matches logical text and maps each matched acronym to its exact source span. Existing rule semantics stay unchanged in this delivery.
- **ACR002:** counts across all blocks and reports the first mapped occurrence for each finding.
- **TERM001:** matches configured terms in logical text and reports the matched term's mapped source span.
- **STYLE001:** segments each block with `segment_sentences`, whose `Sentence` result is extended to carry an exact logical UTF-8 byte range (including correct trim offsets); it detects language with `detect_language`, calculates Chinese/mixed effective length with `calculate_stats`/`effective_length`, and maps the entire offending sentence back to source.

### STYLE001 configuration

The public configuration becomes:

```toml
[rules.STYLE001]
level = "warning"
max_chars = 80
max_english_words = 45
```

`max_chars` applies to Chinese and mixed sentences. `max_english_words` applies to English sentences. The old `max_words` key remains accepted as a serde alias for `max_english_words`; omitted `max_chars` receives the default `80` so an existing complete `STYLE001` table using `max_words` remains valid.

## Human Diagnostics

Add Ariadne `0.6.0` with its automatic-color support. Ariadne is confined to the output layer.

The default human report contains:

```text
warning[STYLE001]: sentence has 112 effective characters; max is 80
  ┌─ chapters/introduction.tex:42:1
  │
42│ ...original LaTeX source...
  │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Found 1 problem: 0 errors, 1 warning
```

Ariadne 0.6 normally places `with_code` before its built-in report kind. To preserve the approved compiler-style `warning[RULE]`/`error[RULE]` header, the adapter uses `ReportKind::Custom("warning[RULE]", Yellow)` or `ReportKind::Custom("error[RULE]", Red)` and does not call `with_code`. All source-frame layout remains Ariadne-owned.

Exact box glyph placement is delegated to Ariadne. Required semantics are:

- `error`: red semantic color;
- `warning`: yellow semantic color;
- rule ID/path: restrained cyan or Ariadne's stable label color;
- Unicode source frame;
- byte-span indexing;
- tab width `4`;
- original source line and underlined primary span;
- one final aggregate summary.

When no diagnostics exist, human output is:

```text
No problems found.
```

Display paths are made relative to the document root when possible and use `/` separators. `Document.root` is the canonical parent directory of the canonical entry file. Stored `PathBuf` values are not rewritten. Renderer-emitted line endings are `\n` and contain no `\r` even when source snapshots contain CRLF.

### Color control

CLI options:

```text
--color auto|always|never
```

- `auto` is the default and enables color only when stdout supports terminal color and color has not been disabled by standard environment controls such as `NO_COLOR`.
- `always` emits ANSI styling even through a pipe.
- `never` emits no ANSI escapes.
- JSON ignores the color option and never emits ANSI.

Use standard 16-color semantics; do not hard-code RGB/TrueColor values.

## CLI Format Compatibility

The canonical formats are:

```text
--format human
--format json
```

The previously public `--format text` value remains an accepted alias for `human`. Existing invocations must continue to work.

## Exit Codes

Exit status is derived from lint data, not the renderer:

- `0`: no error-level diagnostics (including warning-only results);
- `1`: at least one error-level lint diagnostic;
- `2`: CLI/configuration/project/read/parse/rendering operational failure.

Warnings are still printed in full even though they do not fail the process.

## Error Handling

- Missing entry/config/include files, include cycles, Tree-sitter setup/parse failure, source-cache mismatches, human rendering I/O/UTF-8 conversion errors, and final stdout write failures become contextual `PaperlintError` variants and exit `2`.
- Before invoking Ariadne, the renderer validates that every diagnostic path exists in the supplied source snapshots and that every byte range is valid; Ariadne must never silently skip an unavailable label or write a cache warning to stderr.
- No parser or renderer error is silently converted into an empty document/report.
- Diagnostics are sorted deterministically before either renderer (source display path, byte start/end, rule ID, then message); JSON serialization failure is returned rather than silently replaced with `[]`.
- Include errors identify both the including source and unresolved request.

## Testing Strategy

Every production behavior is introduced with a failing regression test first.

### Parser/project tests

Use temporary multi-file projects to prove:

1. `main.tex → chapter.tex → nested.tex` is expanded recursively in reading order.
2. All four `latex_include` commands are recognized from AST nodes.
3. Extensionless includes resolve to `.tex`.
4. Include paths are relative to the including file.
5. A cycle produces a contextual error and terminates.
6. A repeated completed file is linted once.
7. Missing includes produce an operational error.
8. Comments, math, ignored code environments, citations, labels/references, definitions, and package/bibliography/graphics/include resource paths are excluded.
9. Natural-language formatting arguments, section/chapter titles, and captions are retained with whitespace and sentence punctuation intact.
10. Logical matches map to the original child file and exact UTF-8 byte range.
11. Sentence ranges remain exact when the source has leading whitespace before a sentence.

### Chinese rule tests

Prove that:

1. A Chinese sentence over `max_chars` ending in `。` produces `STYLE001`.
2. A shorter Chinese sentence does not.
3. English uses `max_english_words`.
4. Mixed text uses effective-character length.
5. The old `max_words` config alias still parses and uses default `max_chars = 80`.

### Human/CLI tests

With fixed `color=never`, Unicode charset, tab width `4`, `/` display paths, and `\n` renderer newlines, cover:

1. deterministic compiler-style output for Chinese, English, and mixed source;
2. multibyte marker alignment delegated through Ariadne byte indexing;
3. tabs and CRLF input without byte-range drift, with no `\r` in rendered output;
4. exact relative slash-normalized display paths rooted at the canonical entry directory;
5. `--color never` contains no escape sequences;
6. `--color always` contains ANSI sequences;
7. non-TTY `auto` output contains no ANSI sequences;
8. JSON contains no ANSI/presentation fields;
9. `--format text` and `--format human` both work;
10. warning-only exits `0`, error findings exit `1`, operational failures exit `2`;
11. summary counts match diagnostics;
12. multiple findings render and serialize in deterministic order;
13. a diagnostic whose source snapshot is absent fails before Ariadne rendering.

### Final verification

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Then build and run the exact binary against:

```text
~/Data/coding/BioMed-QAgent/docs/latex/main.tex
```

Acceptance requires diagnostics from recursively included chapter files and at least one Chinese `STYLE001` diagnostic where the source exceeds the configured threshold. Human output must show chapter paths and source excerpts; JSON must contain child-file spans and no presentation data.

## Delivery Organization

Implementation is split into two parallel worktree lanes followed by serial integration:

1. **Parser/rules lane:** logical document, AST extraction, recursive includes, source mapping, and Chinese `STYLE001`.
2. **Human-output lane:** Ariadne renderer and output-focused tests against the existing `SourceFile`/`Diagnostic` boundary.
3. **Integration lane:** apply both worktree patches, wire CLI/main, resolve only shared-boundary conflicts, update docs, and run focused integration tests.
4. **Review wave:** two fresh read-only reviewers inspect parser/rule correctness and output/CLI portability independently; one writer applies accepted blocking fixes, followed by focused re-review.

Concurrent writers never share a worktree.
