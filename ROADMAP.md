# Paperlint Roadmap

Paperlint is a deterministic linter for Chinese-first academic writing in LaTeX projects. The roadmap below separates stable mechanical checks from higher-cost language understanding so the core remains fast, explainable, reproducible, and suitable for editor/CI integration.

## Guiding principles

- Prefer deterministic checks when they can solve the problem reliably.
- Keep LaTeX parsing, lexical analysis, terminology resolution, and semantic inference as separate layers.
- Preserve source spans through every stage so every diagnostic maps back to the original file.
- Treat dictionaries and project terminology as data, not hard-coded rule logic.
- Unknown words are candidates, not errors by definition.
- Expensive NLP or learned models must be optional fallbacks rather than mandatory dependencies.
- The same input, configuration, dictionaries, and model version must produce the same diagnostics.

## Target architecture

```text
LaTeX project
    -> ProjectResolver / Parser
    -> Logical Document
    -> Sentence / language segmentation
    -> Lexical Analysis
         - Chinese / English tokenization
         - POS tags
         - logical UTF-8 byte ranges mapped back to source spans
    -> Lexicon & Term Resolution
         - built-in dictionary
         - user dictionary
         - workspace dictionary
         - document-discovered terms / acronyms
    -> Candidate Classification
         - common word
         - term
         - acronym
         - proper noun
         - unit / symbol / identifier
         - unknown candidate
    -> Rule Engine
         - acronym rules
         - case consistency
         - terminology consistency
         - unknown-term diagnostics
         - explanation-required checks
              -> deterministic patterns
              -> syntax features when available
              -> optional semantic classifier
```

The parser should not decide whether a term is valid or explained. It should only construct the logical document and retain accurate source locations. Terminology and semantic judgments belong to later analysis stages.

## Phase 1 — Lexicon infrastructure

**Priority: ✅ delivered (minimal)** — `Lexicon`/`Lexeme`/`LexemeKind` in `src/text/lexicon.rs`; workspace config `[[lexicon.entries]]`; built-in cross-domain core; batch overlay with frozen surface ownership; Aho-Corasick occurrence scan; workspace canonical and alias surfaces are injected into jieba as technical nouns. User-global dictionary layer deferred.

Build a reusable lexicon layer shared by acronym, terminology, spelling, and future explanation rules.

### Goals

- Support three persistent dictionary layers:
  1. built-in dictionary,
  2. user dictionary,
  3. workspace/project dictionary.
- Add a document-local registry populated from definitions found while scanning the paper.
- Define deterministic precedence:

```text
workspace > user > built-in
```

Document declarations are tracked separately because they contain source locations and first-use state.

### Lexeme metadata

Do not model the dictionary as only `HashSet<String>`. Terms need metadata such as:

```rust
Lexeme {
    canonical,
    aliases,
    kind,
    source,
    case_sensitive,
    requires_explanation,
}
```

Suggested kinds:

```text
Common
Term
Acronym
ProperNoun
Unit
Symbol
Unknown
```

The exact Rust representation can evolve, but rules should consume a stable semantic abstraction rather than inspect raw dictionary files directly.

### Tokenizer integration

- Feed relevant project/user terms into the Chinese tokenizer where supported.
- Keep tokenization and dictionary lookup separate: a tokenizer result is evidence, not the definition of what constitutes a term.
- Preserve byte spans for every token and matched term.

### Deliverables

- Lexicon loading and overlay logic.
- Config format for user/workspace entries.
- Canonical names and aliases.
- Case policy.
- Dictionary-aware lexical tests covering Chinese, English, and mixed text.

## Phase 2 — Document Term Registry

**Priority: ✅ delivered (minimal)** — `DocumentAnalysis` + `DocumentTermRegistry` + `LintContext`; paragraph-like parser blocks, sentence language, jieba tokens/POS, logical byte ranges, acronym definitions/usages, and known lexicon occurrences are built once in reading order and shared by rules. Observed-capitalization tracking partially available via CASE001 token walk.

Build a document-scoped symbol/term table while traversing the logical document in reading order.

### Track

- canonical term,
- English name when present,
- acronym,
- declaration span,
- first occurrence,
- later usages,
- aliases,
- observed capitalization variants.

Example:

```text
检索增强生成（Retrieval-Augmented Generation, RAG）
```

should register the relationship between the Chinese term, English full name, and `RAG` rather than leaving each rule to rediscover it independently.

### Expected rule reuse

This registry should become the common data source for:

- ACR001 / ACR002,
- TERM001,
- TERM002,
- capitalization consistency,
- duplicate or conflicting declarations,
- first-use checks.

## Phase 3 — English lexical diagnostics

**Priority: ✅ delivered (minimal)** — CASE001 enforces canonical casing for lexicon-known acronyms/proper nouns/units and document declarations; unknown mixed-case words stay silent; ACR001 consumes lexicon kinds for denoising. English identifier shapes now contribute evidence to the shared term-candidate registry; spelling diagnostics remain deferred.

Introduce English candidate classification without assuming that every unknown token is misspelled.

### Acronym candidates

Treat all-uppercase tokens as `AcronymCandidate`, not automatically as acronyms.

Resolve candidates against:

- known acronyms,
- proper nouns,
- units,
- configured ignores,
- document declarations,
- other lexicon metadata.

Unresolved identifier shapes can contribute shared candidate evidence. No unknown-term diagnostic currently consumes that evidence.

### Case consistency

When a known case-sensitive entry exists, detect variants such as:

```text
PyTorch -> Pytorch / pytorch / PYTORCH
RAG     -> Rag / rag
```

and emit a precise canonical-form suggestion.

### Separate rule classes

Keep these concepts distinct:

- unknown lexical item,
- probable spelling mistake,
- capitalization inconsistency,
- undeclared acronym.

A dictionary miss alone must never prove a spelling error.

## Phase 4 — Term-candidate extraction

**Priority: ✅ delivered (shared analysis)** — `DocumentTermRegistry::term_candidates` exposes byte-exact surfaces in deterministic reading order, with the first logical `LocatedRange`, normalized evidence, a bounded score, and a kind hint. Known lexicon surfaces and exact document declarations are suppressed before candidates are retained.

Chinese unknown-term detection cannot rely only on tokenizer output. A novel technical term may be segmented entirely into common known words.

Example:

```text
跨模态语义蒸馏网络
```

may be segmented as:

```text
跨模态 / 语义 / 蒸馏 / 网络
```

while the full phrase is still the concept that may require explanation.

### Candidate signals

The delivered sentence-local extractor uses deterministic evidence from:

- bounded, repeated 2–6 token Chinese noun/adjective n-grams,
- all-caps, Camel/Pascal, and letter-plus-digit English identifiers,
- bounded Chinese-English mixed runs,
- quoted or parenthesized terms,
- explicit Chinese and English definition-introduction forms.

Project-configured lexicon entries and exact Chinese, English, or acronym declaration surfaces are suppressors, not candidate signals. The goal is candidate discovery, not automatic classification as an error.

### Output

The extractor stores one `TermCandidate` per byte-exact surface in the shared registry. Locations are logical UTF-8 byte ranges; consumers map them to source spans with `LintContext::span_of` (or `Document::source_span` at the lower level). This registry is evidence-only analysis data: no candidate lint rule, configuration block, diagnostic, or JSON field is delivered.

`TERM002` does not consume unknown candidates. It still checks only configured `[[lexicon.entries]]` with `requires_explanation = true`.

## Phase 5 — TERM002 deterministic explanation detection

**Priority: ✅ delivered (minimal)** — configured terms only (`requires_explanation = true`), first-occurrence inspection, Chinese/English trigger patterns, structured-declaration exemption. Evaluation corpus (Phase 7) still pending before any model work.

Implement the first reliable version of "term needs explanation on first use" without a learned model.

### Initial patterns

Support common Chinese and English definition forms such as:

```text
X 是……
X 指……
X 是指……
X 指的是……
所谓 X……
X，即……
X 定义为……
称 X 为……
X（English Name，ABC）
X (also known as ...)
X refers to ...
X is defined as ...
where X denotes ...
```

### Behavior

- Inspect only the first relevant occurrence by default.
- Search a configurable local context window.
- Prefer structured evidence from brackets, punctuation, and syntax rather than raw substring matching.
- Report warnings, not hard errors.
- Allow projects to mark terms as common knowledge or exempt from explanation.

This stage should establish an evaluation corpus before any neural model is introduced.

## Phase 6 — Syntax-aware rules

**Priority: medium term**

Continue the planned syntax backend work for rules that cannot be handled reliably through token/POS heuristics.

Potential consumers include:

- complex 把 / 被 constructions,
- missing sentence components,
- modifier attachment,
- dependency depth,
- subject-verb / verb-object mismatch,
- explanation detection features.

The syntax backend must remain optional. Rules that require dependency analysis should skip cleanly when the backend is disabled.

## Phase 7 — Explanation detection evaluation dataset

**Priority: before introducing a semantic model**

Create a dedicated dataset for:

```text
(term, local context) -> explained / not explained
```

### Data collection

- sample Chinese and English academic papers,
- extract likely technical terms and first-use contexts,
- use an LLM for large-scale preliminary labeling if useful,
- manually review validation samples,
- maintain a fully human-reviewed test set.

### Important hard negatives

The dataset must distinguish definitions from ordinary usage or claims:

```text
本文采用 X 进行……              -> not necessarily explained
X 能够有效提升模型性能。        -> not explained
与传统 X 不同……                 -> assumes prior knowledge
```

The test set should contain many such cases because they define whether a semantic fallback adds real value over rules.

## Phase 8 — Optional semantic explanation classifier

**Priority: long term; only if evaluation justifies it**

Introduce a learned classifier only for candidates that deterministic logic cannot resolve confidently.

Target abstraction:

```text
f(term, context) -> P(explained)
```

### Preferred progression

Start with lightweight feature-based models before a Transformer:

- logistic regression,
- small MLP,
- compact tree-based classifier,
- features derived from lexical/syntax analysis.

Candidate features include:

- term position,
- same-sentence relationship,
- brackets / colon / punctuation,
- definition trigger words,
- copula presence,
- dependency relations,
- token distance,
- POS context,
- occurrence count.

If these models cannot solve the remaining ambiguous cases, evaluate a very small bilingual encoder exported to ONNX.

### Runtime constraints

The model must remain optional and local.

Initial engineering targets, subject to measurement:

```text
warm P50 < 10 ms / candidate
warm P95 < 25 ms / candidate
32-candidate batch < 100 ms
additional latency for a typical paper < 200 ms
```

The final target should be based on end-to-end lint latency, not an arbitrary single-inference number.

Use batching where useful. Do not run inference for candidates already resolved by deterministic rules.

### Caching

If model inference becomes significant, cache using all inputs that affect semantics, for example:

```text
hash(
    model_version,
    preprocessing_version,
    term,
    normalized_context,
    threshold
)
```

Changing the model or preprocessing must invalidate old results.

## Longer-term rule opportunities

Once the shared lexical/term infrastructure is stable, Paperlint can add rules without duplicating parsing logic:

- duplicate/conflicting acronym definitions,
- term alias drift across chapters,
- deprecated terminology,
- unexplained named methods/models,
- project-specific required terminology,
- suspicious English spelling candidates,
- CJK/Latin spacing and punctuation consistency,
- cross-reference terminology checks across included files.

## Non-goals

Paperlint should not become a general prose-generation system or judge whether writing is "beautiful". Its strongest niche is precise, mechanical diagnostics that authors can understand and reproduce.

Unknown-term diagnostics remain intentionally undesigned and have no assigned rule ID. In particular, the delivered `TERM002` rule is not an unknown-candidate rule; it checks only configured lexicon entries that explicitly require an explanation.

## Priority summary

| Area | Priority |
| --- | --- |
| Lexicon infrastructure | ✅ Delivered (minimal) |
| Document Term Registry | ✅ Delivered (minimal) |
| English acronym/case classification | ✅ Delivered (CASE001) |
| Deterministic term-candidate extraction | ✅ Delivered (shared analysis only) |
| Unknown-candidate diagnostics | Not delivered |
| Chinese compound candidate evidence | ✅ Delivered (bounded repeated n-grams) |
| Deterministic TERM002 | ✅ Delivered (minimal) |
| Dependency/syntax backend | Medium term |
| Explanation evaluation dataset | Before model work |
| Optional semantic classifier / ONNX | Long term |
| LLM-assisted training-data generation | Model phase only |

The shared **Logical Document -> Lexical Analysis -> Document Term Registry -> Rules** pipeline now carries deterministic candidate evidence without changing diagnostics. The next decision is how that evidence should be evaluated before any separate unknown-term rule or optional model work is designed.
