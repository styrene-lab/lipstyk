# Python Human Precision Corpus v1

This corpus measures false positives on established human-authored Python. It is
not an authorship-classification benchmark and contains no agent cohort.

## Construction

The corpus contains 24 files: four files from each of CPython, Flask, Click,
Requests, pytest, and NumPy. Revisions are immutable release commits no later
than May 2021, before widespread code-generating LLM use. Categories include
libraries, applications, command-line tools, tests, examples, scientific code,
and benchmarks.

`import.lock.json` records each repository, commit, original path, category,
content SHA-256, and local object name. The selected source files are checked in
under `objects/`; their upstream licenses are recorded per sample in
`corpus.json`. Selection was manual and stratified for diversity, so this corpus
is reproducible but not statistically representative of all Python.

Run the sweep with:

```bash
cargo run --bin lipstyk-eval -- \
  evaluation/corpus/python-human-precision-v1/corpus.json
```

## Current results

At the unchanged threshold of `1.0/100 lines`, detector version 0.2.1 produced
78 quality diagnostics across 13 files, but **0 generation-channel diagnostics**
and therefore 0/24 agent predictions under the separated scoring model. The
legacy all-diagnostic aggregate still has median 1.09, mean 2.55, p95 12.77,
and maximum 16.88 per 100 lines; it is retained for compatibility and quality
reporting, not authorship classification.

### Per-rule attribution

| Rule | Diagnostics | Assessment |
|---|---:|---|
| `py-trivial-wrapper` | 44 | Dominant false-positive source. Protocol adapters, decorators, tests, and benchmark methods legitimately contain many one-statement functions. |
| `py-structural-repetition` | 14 | Frequently legitimate in protocol families, tests, and homogeneous benchmark classes. |
| `py-nesting-depth` | 8 | General maintainability signal, not generation-specific evidence. |
| `py-restating-comment` | 4 | All four are false positives on intent/context comments; none came from the new repeated-inline-narration branch. |
| `bare-except` | 2 | Intentional compatibility behavior in CPython's completer; quality warning, not authorship evidence. |
| `type-hint-gaps` | 2 | Expected during gradual typing and therefore weak authorship evidence. |
| `py-naming-entropy` | 1 | False positive on CPython's queue implementation and repeated protocol method vocabulary. |
| `py-comment-depth` | 1 | False positive on dense synchronization-invariant comments in CPython queue initialization. The new imperative-narration branch did not fire. |
| `import-star` | 1 | Intentional public terminal API import in CPython. |
| `print-debug` | 1 | Intentional subprocess output in NumPy's pytest helper. |

The three recently calibrated Python systems behaved conservatively here:

- `py-demo-scaffolding`: 0 findings;
- `py-placeholder-scaffolding`: 0 findings;
- repeated inline narration in `py-restating-comment`: 0 findings.

The imperative-comment addition to `py-comment-depth` also produced no finding;
the sole `py-comment-depth` hit came from its older function-density logic.

## Consequences

`total_score` / `score` remain compatibility aggregates across all diagnostics.
Machine-readable reports now also expose independent `channel_scores.quality`
and `channel_scores.generation` values, each diagnostic's `channel`, and a
per-file `generation_score_per_100_lines`. Evaluation classification uses only
the generation channel; quality findings remain visible but cannot produce an
agent prediction. Generation-channel admission is deliberately narrow and
currently limited to the calibrated Python demo scaffolding, placeholder
scaffolding, imperative-comment narration, and repeated inline narration
signals.

Do not tune the global threshold to this corpus: doing so would hide useful
quality findings while preserving category confusion. The next detector design
step should separate general quality rules from generation-associated evidence
and report both channels independently. In particular, trivial wrappers,
structural repetition, nesting, broad exceptions, and type-hint consistency
should not contribute to an authorship-like aggregate without independent
validation.
