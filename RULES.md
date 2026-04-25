# Lipstyk Rule Reference

77 rules across 10 file types. Every finding is deterministic — no ML,
no classifiers.

Severity levels:
- **Hint** (0.5–1.0) — Could be human, suspicious in aggregate
- **Warning** (1.0–2.0) — Likely slop pattern
- **Slop** (2.0–3.0) — Strong indicator of machine-generated code

Any single finding is inconclusive. Density is the signal.

---

## Rust (21 rules, AST via `syn`)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `unwrap-overuse` | Dense `.unwrap()` / `.expect()` | H→S | 0.1→3.0 |
| `error-swallowing` | `Err(_) => {}`, `.unwrap_or_default()` on Results | H→S | 0.75→2.5 |
| `boxed-error` | 2+ functions returning `Box<dyn Error>` | W | 1.5 |
| `redundant-clone` | `.clone()` that could borrow | H→S | 0.5→1.5 |
| `string-params` | `fn foo(s: String)` instead of `&str` | W | 1.5 |
| `needless-lifetimes` | Lifetimes the elision rules handle | H | 0.75 |
| `needless-type-annotation` | `let x: Vec<String> = Vec::new()` | H | 0.5 |
| `verbose-match` | 2-arm match replaceable with `if let` / `.map()` | W | 1.0 |
| `index-loop` | `for i in 0..vec.len()` | W | 1.5 |
| `restating-comment` | Comments restating the next line of code | W | 1.5 |
| `over-documentation` | Step-by-step tutorial narration, >45% density | W→S | 2.0→3.0 |
| `comment-clustering` | Per-function comment density >50%, uniform spacing | W→S | 1.5→2.5 |
| `generic-todo` | Vague TODOs: "add error handling" | W | 1.5 |
| `generic-naming` | `process_data`, `handle_event` | W | 1.5 |
| `naming-entropy` | Low unique-stem ratio (<35%) | H→W | 0.75→1.5 |
| `structural-repetition` | 3+ functions with identical AST shape | W | 1.5 |
| `whitespace-uniformity` | Regular blank line spacing, uniform line lengths | H | 1.0 |
| `trivial-wrapper` | Single-expression delegators (6+/file, 15+ for orchestration) | H | 0.75 |
| `pub-overuse` | >70% of items `pub` (exempts types.rs, data models) | W | 1.5 |
| `dead-code-markers` | 3+ `#[allow(dead_code)]` | W | 1.5 |
| `derive-stacking` | 6+ derives on one type | H | 0.75 |

Escalation: `redundant-clone` at >15 = Warning, >30 = Slop.
`unwrap-overuse` downweighted to 0.1 in test code, suppressed with `--exclude-tests`.

---

## TypeScript / JavaScript (14 rules, AST via `oxc`)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `any-abuse` | 3+ `any` types or `@ts-ignore` suppressions | W→S | 1.5→3.0 |
| `ts-error-handling` | Empty catch blocks, catch-and-log-only, catch(_) | W→S | 1.5→2.5 |
| `promise-antipattern` | `new Promise`, `.then()` chains, silent `.catch()` | H→S | 0.75→2.5 |
| `console-dump` | 3+ debug `console.*` calls | W→S | 1.5→3.0 |
| `ts-redundant-async` | `async` functions that never `await` | W | 1.0 |
| `nested-ternary` | 2+ ternary operators on one line | W | 1.5 |
| `ts-nesting-depth` | 4+ levels of nested control flow | W | 1.5 |
| `ts-trivial-wrapper` | 5+ single-statement functions in one file | H | 0.75 |
| `ts-structural-repetition` | 3+ functions with identical shape | W | 1.5 |
| `ts-naming-entropy` | Low identifier uniqueness ratio | W | 1.5 |
| `ts-generic-naming` | `processData`, `handleRequest`, `fetchData` | W | 1.5 |
| `ts-restating-comment` | Comments restating code | W | 1.5 |
| `ts-comment-depth` | Per-function density, step narration | W→S | 1.5→3.0 |
| `ts-whitespace-uniformity` | Uniform blank lines and line lengths | H | 1.0 |

`ts-error-handling` uses oxc's typed `CatchClause` to detect empty bodies
and log-only patterns. `ts-redundant-async` recurses into try/if blocks
to find await expressions.

---

## Python (15 rules, AST via `tree-sitter`)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `bare-except` | `except:` or `except Exception:` | H→S | 0.75→2.5 |
| `py-error-handling` | Broad except + pass, except + log-only | W→S | 1.5→2.5 |
| `print-debug` | 3+ `print()` in non-CLI code | W→S | 1.5→3.0 |
| `import-star` | `from X import *`, 20+ imports | H→W | 0.75→1.5 |
| `type-hint-gaps` | 20-80% type hint coverage | H | 1.0 |
| `py-index-loop` | `for i in range(len(x))` | W | 1.5 |
| `py-mutable-default` | `def f(x=[])` | W | 1.5 |
| `py-nesting-depth` | 4+ levels of nested control flow | W | 1.5 |
| `py-trivial-wrapper` | 5+ single-statement functions | H | 0.75 |
| `py-structural-repetition` | 3+ functions with identical shape | W | 1.5 |
| `py-naming-entropy` | Low identifier uniqueness | W | 1.5 |
| `py-generic-naming` | `process_data`, `handle_request` | W | 1.5 |
| `py-restating-comment` | Comments restating code | W | 1.5 |
| `py-comment-depth` | Per-function density, step narration | W→S | 1.5→3.0 |
| `py-whitespace-uniformity` | Uniform blank lines and line lengths | H | 1.0 |

---

## Go (8 rules, AST via `tree-sitter` + custom collector)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `go-error-handling` | Bare `return err`, `panic()` in library code, ignored errors | W→S | 1.5→2.5 |
| `go-antipattern` | `interface{}` overuse, `fmt.Print` debugging, `time.Sleep` | W→S | 1.5→2.5 |
| `go-nesting-depth` | 4+ levels of nested control flow | W | 1.5 |
| `go-structural-repetition` | 3+ functions with identical shape | W | 1.5 |
| `go-naming-entropy` | Low identifier uniqueness | W | 1.5 |
| `go-generic-naming` | `processData`, `handleRequest` | W | 1.5 |
| `go-restating-comment` | Comments restating code | W | 1.5 |
| `go-comment-depth` | Per-function density, step narration | W→S | 1.5→3.0 |

Go AST collector understands error return types, method receivers,
`interface{}` nodes, and function nesting depth.

---

## HTML / CSS (6 rules, tag parser)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `div-soup` | >50% div tags or 5+ nesting levels | W→S | 2.5→3.0 |
| `missing-semantics` | Zero semantic elements in 15+ tag file | W | 2.0 |
| `inline-styles` | 3+ inline `style=""` attributes | W→S | 1.5→3.0 |
| `generic-classes` | `container`, `wrapper`, `content` | W | 1.5 |
| `accessibility` | Missing alt, lang, aria-label | W→S | 1.0→3.0 |
| `css-smells` | `!important`, magic numbers, no custom properties | H→S | 1.0→3.0 |

---

## Java (4 rules, text, legacy)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `java-bare-catch` | `catch (Exception e)` with empty/log-only body | W | 1.5 |
| `java-generic-naming` | Generic method names | W | 1.5 |
| `java-restating-comment` | Comments restating code | W | 1.5 |
| `java-comment-depth` | Per-function density, step narration | W→S | 1.5→3.0 |

---

## Shell (3 rules, text)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `sh-strict-mode` | Missing `set -euo pipefail`, missing/wrong shebang | W | 1.5→2.0 |
| `sh-unquoted-var` | Unquoted `$VAR` expansions | W→S | 1.5→2.5 |
| `sh-antipattern` | `cat \| grep`, parsing `ls`, unchecked `cd`, `eval`, hardcoded `/tmp` | H→W | 0.75→1.5 |

---

## Dockerfile (1 rule, 5 checks)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `docker-best-practices` | No USER (root), :latest tag, split RUN layers, apt without cleanup, ADD vs COPY | H→S | 0.75→2.5 |

---

## Kubernetes YAML (1 rule, 6 checks, content-sniffed)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `k8s-manifest` | No resource limits, no probes, naked pods, default namespace, :latest image, wildcard RBAC | H→S | 1.0→2.5 |

Only fires on YAML containing `apiVersion:` and `kind:`.

---

## CI/CD YAML (1 rule, 5 checks, content-sniffed)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `ci-workflow` | Hardcoded secrets, wildcard triggers, auto-approve, missing permissions, unpinned actions | H→S | 1.0→3.0 |

Only fires on YAML containing `jobs:` + `steps:` (GHA) or `stages:` (GitLab).

---

## Markdown (3 rules, text)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `md-slop-phrases` | AI buzzword density (comprehensive, leverage, delve, etc.) | W→S | 1.5→2.5 |
| `md-structure` | H5+ heading depth, uniform sub-heading structure | H→W | 0.75→1.5 |
| `md-placeholder` | Template filler, generic opening paragraphs | H→W | 1.0→1.5 |

---

## Cross-file (3 rules)

| Rule | What It Catches | Sev | Weight |
|------|----------------|-----|--------|
| `cross-file-duplicate` | 5-line blocks duplicated across 3+ files | W | 2.0 |
| `cross-file-imports` | Identical import headers across 3+ files | H | 1.0 |
| `cross-file-error-pattern` | Same error handling in 3+ files | W | 1.5 |

These run after per-file analysis completes.

---

## Research Basis

Rule design draws from published work on detecting machine-generated code.

**Comment-to-code ratio is the most reliable surface signal.** The
CoDet-M4 multi-language study and multilingual code stylometry work
both found this as the single strongest surface discriminator.

**Function-level analysis far outperforms file-level.** This motivated
per-function density measurement in `comment-clustering` and the
language-specific `*-comment-depth` rules.

**AI distributes comments uniformly; humans cluster.** This backs
the standard-deviation analysis in `comment-clustering` and
`whitespace-uniformity`.

**Structural signals survive obfuscation.** AST structure, control flow
patterns, and dataflow graphs are more robust than surface features.
This validates `structural-repetition` and the tree-sitter/oxc AST
analysis approach.

**Naming entropy separates human from AI code.** AI draws from a
narrow naming vocabulary.

**AI coding agents leave distinct fingerprints.** The MSR 2026 agent
fingerprinting study (33k PRs) achieved 97% F1 on agent identification.

**Detection degrades with each model generation.** Surface rules
degrade faster than structural rules. The rule set must evolve.

### Sources

- CoDet-M4: Multi-Lingual, Multi-Generator Detection (ACL Findings)
- Fingerprinting AI Coding Agents on GitHub (MSR 2026, arxiv 2601.17406)
- The Hidden DNA of LLM-Generated JavaScript (arxiv 2510.10493)
- Code Fingerprints: Disentangled Attribution via DCAN (arxiv 2603.04212)
- AICD Bench: 2M-sample benchmark, 77 generators, 9 languages (arxiv 2602.02079)
- Multilingual Code Stylometry (SANER — 84.1% across 10 languages)

---

## Scoring

Each diagnostic has a weight. File score = sum of weights.
`score_per_100_lines` normalizes for size.

Rules escalate by count: one `.clone()` is a 0.5 hint; fifteen+
escalates to 1.0 warning; thirty+ escalates to 1.5 slop.

Verdicts: clean (<5), mild (<15), suspicious (<30), sloppy (>=30).

---

## Limitations

Detects patterns, not intent. False positives on verbose human code.
False negatives on edited AI output. Not a replacement for reading
the code.
