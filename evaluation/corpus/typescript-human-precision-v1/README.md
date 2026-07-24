# TypeScript Human Precision Corpus v1

This corpus measures false positives on established human-authored TypeScript.
It is not an authorship benchmark and contains no agent cohort.

## Construction

The corpus contains 24 files: four each from TypeScript, VS Code, Angular,
NestJS, Redux, and TypeORM. Every repository is pinned to an immutable commit no
later than May 2021. The manually stratified selection covers compiler and
language-service internals, utilities, asynchronous infrastructure,
configuration, frontend components, framework routing and dependency injection,
state management, query building, database drivers, decorators, integrations,
and examples.

`import.lock.json` records repository, commit, upstream path, category, license,
content SHA-256, and local object name. The selected source files are checked in
under `objects/`. Selection is reproducible but not statistically representative
of all TypeScript.

Run the sweep with:

```bash
cargo run --bin lipstyk-eval -- \
  evaluation/corpus/typescript-human-precision-v1/corpus.json
```

## Current results

At the unchanged slop threshold of `1.0/100 lines`:

- 24 files evaluated with no errors;
- 0 slop-channel findings and 0 files exceeding the slop threshold;
- 4 files had no findings in either channel;
- the compatibility all-diagnostic aggregate had median 1.35, mean 1.95,
  p95 5.07, and maximum 6.82 per 100 lines;
- general quality findings remain common and must not be treated as strong slop
  evidence.

The initial sweep exposed one slop-channel false positive in TypeScript's
language service: three `// Check if ...` comments preceding `if` branches were
mistaken for declaration narration. `ts-restating-comment` now excludes control
flow before applying declaration-narration logic. The fix preserved the two
existing slop findings in the clean AICD TypeScript calibration cohort while
reducing this corpus to zero slop findings.

## Quality-channel attribution

| Rule | Diagnostics | Assessment |
|---|---:|---|
| `nested-ternary` | 123 | Dominated by mature compiler/query-builder expression code; quality signal, not strong slop evidence. |
| `ts-restating-comment` | 31 | Generic comment matcher remains noisy on contextual comments, but these findings stay quality-only. |
| `ts-trivial-wrapper` | 27 | Common in adapters, framework APIs, and state helpers. |
| `any-abuse` | 17 | Common in older compiler/framework internals and migration boundaries. |
| `ts-comment-depth` | 15 | Dense comments frequently document invariants and compatibility behavior. |
| `promise-antipattern` | 9 | Requires separate precision review; no slop-channel weight. |
| Other quality rules | 16 | Structural repetition, naming, fixed delay, and console output. |

## Consequences

The two TypeScript slop rules currently supported by clean calibration remain
conservative:

- `ts-placeholder-scaffolding`: 0/24 human precision findings and 2/16
  agent-cohort calibration findings;
- repeated declaration narration: 0/24 human precision findings after excluding
  control-flow comments, and no clean AICD coverage currently.

Future slop rules must improve coverage on the pinned calibration cohort while
remaining at zero findings on this corpus. Quality-channel density alone is not
a reason to promote a rule into the slop channel.
