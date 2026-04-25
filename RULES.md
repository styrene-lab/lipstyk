# Lipstyk Rule Reference

Lipstyk detects machine-generated code patterns through static analysis.
It does not use ML classifiers or language models — every finding is a
deterministic rule you can read, understand, and disagree with.

The rules are organized by language. Each rule fires at one of three
severity levels:

- **Hint** (0.5–1.0 weight) — Could be human, but suspicious in aggregate
- **Warning** (1.0–2.0 weight) — Likely slop pattern
- **Slop** (2.0–3.0 weight) — Strong indicator of machine-generated code

Any single finding is inconclusive. The aggregate density is the signal.

---

## Rust (21 rules)

### Code Quality Signals

These catch patterns where AI takes shortcuts a human wouldn't.

| Rule | What It Catches | Sev | Weight | Why It's Slop |
|------|----------------|-----|--------|---------------|
| `redundant-clone` | `.clone()` calls that could borrow | H→S | 0.5→2.0 | AI defaults to cloning rather than reasoning about lifetimes. Escalates: >5=W, >10=S |
| `error-swallowing` | `Err(_) => {}`, `.unwrap_or_default()` on Results | H→S | 0.75→2.5 | AI drops errors silently. Empty catch blocks are the strongest signal |
| `unwrap-overuse` | Dense `.unwrap()` / `.expect()` | H→S | 0.1→3.0 | AI sprinkles unwrap rather than propagating. Multiple on one line = S |
| `boxed-error` | 2+ functions returning `Box<dyn Error>` | W | 1.5 | AI uses the laziest error type instead of defining domain errors |
| `string-params` | `fn foo(s: String)` instead of `&str` | W | 1.5 | AI takes owned Strings everywhere rather than borrowing |
| `index-loop` | `for i in 0..vec.len()` | W | 1.5 | AI writes C-style loops instead of idiomatic iterators |
| `verbose-match` | 2-arm match replaceable with `if let` / `.map()` | W | 1.0 | AI writes match on Option/Result when a combinator is cleaner |
| `needless-lifetimes` | Lifetime annotations the elision rules handle | H | 0.75 | AI writes lifetimes it saw in training data rather than letting the compiler elide |
| `needless-type-annotation` | `let x: Vec<String> = Vec::new()` | H | 0.5 | AI over-annotates types the compiler infers |

### Documentation Signals

These catch patterns where AI narrates rather than explains.

| Rule | What It Catches | Sev | Weight | Why It's Slop |
|------|----------------|-----|--------|---------------|
| `over-documentation` | Step-by-step tutorial comments, >45% comment density | W→S | 2.0→3.0 | AI narrates code like a tutorial: "Step 1: Initialize..." |
| `restating-comment` | Comments that restate the next line of code | W | 1.5 | AI explains obvious code. Heuristic: >60% of comment words appear in code line below. Exempts comments with intent signals (because, workaround, hack, etc.) |
| `comment-clustering` | Per-function comment density >50%, or uniformly spaced comments | W→S | 1.5→2.5 | Research-backed: comment-to-code ratio is the universal discriminator (CoDet-M4, SANER 2025). AI distributes comments uniformly; humans cluster around complexity |
| `generic-todo` | Vague TODOs: "add error handling", "implement this" | W | 1.5 | AI leaves placeholder TODOs it has no plan to fill. 44 match patterns |

### Structural Signals

These catch patterns in how AI organizes code.

| Rule | What It Catches | Sev | Weight | Why It's Slop |
|------|----------------|-----|--------|---------------|
| `structural-repetition` | 3+ functions with identical AST shape | W | 1.5 | AI generates copy-paste variants. Hashes: param count, stmt count, control flow |
| `whitespace-uniformity` | Suspiciously regular blank line spacing or line lengths | H | 1.0 | AI produces mechanically uniform formatting. Human code: stddev >3.0 for gaps, CV >0.6 for line lengths |
| `naming-entropy` | Low unique-stem ratio (<35%), uniformly verbose naming | H→W | 0.75→1.5 | AI reuses the same vocabulary (process_, handle_, user_). Humans abbreviate, use domain shorthand, and vary style |
| `trivial-wrapper` | Single-expression functions that just delegate | H | 0.75 | AI generates indirection layers. Threshold: 5/file (10 for API surface files) |
| `generic-naming` | Vague names: `process_data`, `handle_event` | W | 1.5 | AI picks names from training distribution. 44 prefix/suffix/exact patterns |
| `pub-overuse` | >70% of items are `pub` | W | 1.5 | AI makes everything public rather than designing visibility |
| `dead-code-markers` | 3+ `#[allow(dead_code)]` suppressions | W | 1.5 | AI generates unused code and silences the compiler |
| `derive-stacking` | 6+ derives on one type | H | 0.75 | AI stacks every derive it knows |

---

## HTML / CSS (6 rules)

| Rule | What It Catches | Sev | Weight | Why It's Slop |
|------|----------------|-----|--------|---------------|
| `div-soup` | Excessive `<div>` nesting (>50% of tags or 5+ levels) | W→S | 2.5→3.0 | AI wraps everything in divs instead of using semantic elements |
| `missing-semantics` | Files with many tags but zero semantic HTML | W | 2.0 | AI doesn't think about document structure |
| `inline-styles` | 3+ inline `style=""` attributes | W→S | 1.5→3.0 | AI puts styles inline rather than using classes |
| `generic-classes` | Generic CSS class names (container, wrapper, content) | W | 1.5 | AI picks names from training frequency, not domain |
| `accessibility` | Missing alt, lang, aria-label | W→S | 1.0→3.0 | AI skips accessibility because training data often omits it |
| `css-smells` | Excessive !important, magic numbers, no custom properties | H→S | 1.0→3.0 | AI writes CSS that works but doesn't compose |

---

## TypeScript / JavaScript (7 rules)

| Rule | What It Catches | Sev | Weight | Why It's Slop |
|------|----------------|-----|--------|---------------|
| `any-abuse` | 3+ `any` types or `@ts-ignore` suppressions | W→S | 1.5→3.0 | AI uses `any` to make TypeScript shut up |
| `console-dump` | 3+ debug `console.*` calls left in code | W→S | 1.5→3.0 | AI leaves debug instrumentation in place |
| `nested-ternary` | 2+ ternary operators on one line | W | 1.5 | AI nests ternaries where a function or match would be clearer |
| `promise-antipattern` | `new Promise`, `.then()` chains, silent `.catch()` | H→S | 0.75→2.5 | AI writes Promise patterns from pre-async/await training data |
| `generic-naming` | Generic function/variable names | W | 1.5 | Same as Rust: training distribution names, not domain names |
| `restating-comment` | Comments that restate code without intent | W | 1.5 | Same as Rust: "what" comments instead of "why" comments |
| `whitespace-uniformity` | Suspiciously regular blank line spacing or line lengths | H | 1.0 | Same as Rust: mechanically uniform formatting |

---

## Python (7 rules)

| Rule | What It Catches | Sev | Weight | Why It's Slop |
|------|----------------|-----|--------|---------------|
| `bare-except` | Bare `except:` or broad exception handling | H→S | 0.75→2.5 | AI catches everything and does nothing with it |
| `print-debug` | 3+ `print()` calls as debugging | W→S | 1.5→3.0 | AI uses print where logging or debugger belongs |
| `import-star` | `from X import *` or 20+ imports | H→W | 0.75→1.5 | AI imports everything rather than being specific |
| `type-hint-gaps` | Inconsistent type hints (20–80% coverage) | H | 1.0 | AI partially annotates rather than being consistent |
| `generic-naming` | Generic function names | W | 1.5 | Same training-distribution naming issue |
| `restating-comment` | Comments restating code | W | 1.5 | Same "what not why" pattern |
| `whitespace-uniformity` | Suspiciously regular blank line spacing or line lengths | H | 1.0 | Same as Rust: mechanically uniform formatting |

---

## Research Basis

The rule set is informed by academic research on detecting
machine-generated code. Key findings that shaped rule design:

**Comment-to-code ratio is the universal discriminator.** Every
multi-language detection study (CoDet-M4 at ACL 2025, SANER 2025
multilingual stylometry, the function/class granularity study) found
this as the single most reliable signal. This backs `over-documentation`,
`restating-comment`, and `comment-clustering`.

**Granularity matters 8.6x more than model differences.** Function-level
analysis is dramatically more discriminative than file-level analysis.
This motivated `comment-clustering`'s per-function density measurement
rather than relying solely on file-level `over-documentation`.

**AI distributes comments uniformly; humans cluster.** Research on
comment placement shows AI produces mechanically regular comment
spacing. Humans cluster comments near complex or non-obvious code.
This backs `comment-clustering`'s standard-deviation analysis and
`whitespace-uniformity`'s gap regularity check.

**Naming diversity separates human from AI code.** AI models draw
from a narrow, verbose naming vocabulary consistent with training
data distributions. Human code has higher naming entropy: abbreviations,
domain shorthand, single-letter iterators, and mixed conventions.
This backs `naming-entropy` and `generic-naming`.

**Structural regularity persists through obfuscation.** The JavaScript
attribution study found that dataflow and AST patterns survive even
minification and identifier mangling (90%+ accuracy on mangled code).
This validates `structural-repetition` and suggests future work on
control-flow diversity and dataflow regularity analysis.

**Newer models are harder to detect.** Detection accuracy dropped from
0.96 AUC-ROC (GPT-3.5) to 0.68 (Claude 3 Haiku). Rule-based detection
has a shelf life — patterns that catch 2024-era models may not catch
2026-era output. This means the rule set must evolve, and transparency
(every rule is readable and explainable) is a feature, not a limitation.

### Sources

- CoDet-M4: Multi-Lingual, Multi-Generator Detection — ACL 2025
- Multilingual Code Stylometry — SANER 2025 (84.1% across 10 languages)
- Structural Patterns in LLM-Generated JavaScript — AST/dataflow study
- Function vs Class Granularity Detection — comment ratio as universal discriminator
- DetectCodeGPT — ICSE 2025 (zero-shot detection)
- LLM Code Stylometry for Authorship Attribution — ACM AISec 2025

---

## Scoring

Each diagnostic has a weight. A file's slop score is the sum of all
diagnostic weights. The `score_per_100_lines` metric normalizes for
file size.

Severity escalation: many rules escalate severity based on count
within a file. One `.clone()` is a Hint (0.5); ten is Slop (2.0).
One `any` type is fine; three is a Warning; six is Slop. This reflects
the core principle: any single pattern is inconclusive; density is
the signal.

---

## Limitations

This tool detects patterns, not intent. It will produce:

- **False positives** on human code that happens to be verbose,
  uniformly formatted, or heavily commented (legitimate in some
  contexts)
- **False negatives** on AI code that has been manually edited,
  refactored, or generated by models trained to avoid these patterns

It is not a substitute for code review. It is a signal that something
warrants closer inspection.
