# Evaluation Harness

Lipstyk's evaluation harness runs the current detector against versioned,
labeled production-code corpora. It is deliberately separate from rule unit
tests: unit tests prove that matchers behave as specified; corpora measure
whether the complete detector separates useful cohorts without unacceptable
false positives.

## Run the smoke corpus

```bash
cargo run --bin lipstyk-eval -- evaluation/corpus/smoke/corpus.json
```

The command emits machine-readable JSON containing detector and corpus
revisions, per-sample scores, the confusion matrix, precision, recall,
specificity, and accuracy. `--threshold N` temporarily overrides the manifest
threshold without changing corpus provenance.

The checked-in `smoke` corpus contains constructed examples. It verifies the
pipeline only and **must not be quoted as detector accuracy**.

## Import a pinned external corpus

External datasets are described by small source specifications under
[`sources/`](sources/). The importer downloads only the configured row window,
selects each label/language stratum by a stable SHA-256 ordering, and writes a
corpus plus an `import.lock.json` containing upstream row IDs and content
hashes:

```bash
cargo run --bin lipstyk-eval -- import \
  evaluation/sources/aicd-t2-validation.source.json
```

A source revision must be an immutable 40-character commit SHA. The `output` directory is resolved beneath the source specification directory;
generated artifacts are content-addressed, and rerunning the same source
specification and upstream revision produces the same selection. The AICD
source specification is intentionally small and records its unresolved
per-sample licensing status. Do not commit imported source objects until
redistribution rights have been verified. Commit the source specification and
lock metadata; keep large or license-unclear generated corpus directories out
of release artifacts.

`rows_url` exists only to support an explicit alternate dataset-server mirror
and hermetic integration tests. Production source specifications should leave
it `null` so the importer constructs the Hugging Face rows endpoint.

## Corpus v1 contract

The authoritative machine-readable schema is
[`schema/corpus-v1.schema.json`](schema/corpus-v1.schema.json). Every manifest
contains:

- `schema_version`: currently `1`;
- immutable corpus identity and revision;
- the score quantity and candidate threshold under evaluation;
- samples labeled `human`, `agent`, `mixed`, or `unknown`;
- an evaluation split (`calibration`, `validation`, or `test`);
- one or more relative artifacts and their semantic roles;
- provenance, source revision, license, collection date, and optional generator
  metadata.

Artifact paths resolve relative to the manifest and may not escape that
directory. This keeps a corpus portable and prevents a manifest from reading
arbitrary host files.

Evaluation reports include score distributions grouped by language and label:
`samples`, `zero_scores`, minimum, p25, median, p75, p95, maximum, and mean.
These distributions expose detector coverage and overlap that aggregate accuracy
hides.

### Current pinned calibration evidence

`evaluation/sources/aicd-t2-calibration.source.json` deterministically selects
109 AICD-Bench T2 training samples: 25 human and 25 agent samples for each of
Python and Java, plus the nine Go rows available in the first 1,000 records
(4 human and 5 agent). The importer reads the revision-pinned Parquet shards
directly and caches them
under `.cache/aicd-t2-parquet`, avoiding the paginated Hugging Face Dataset
Server and its anonymous HTTP 429 limits. Rebuild the corpus with:

```bash
cargo run --bin lipstyk-eval -- import \
  evaluation/sources/aicd-t2-calibration.source.json
cargo run --bin lipstyk-eval -- \
  evaluation/sources/aicd-t2-calibration/corpus.json
```

At the existing `1.0/100 lines` threshold, this calibration slice produced 5
true positives, 8 false positives, 42 true negatives, and 45 false negatives
(10% recall, 84% specificity). More importantly, every group had a median score
of zero; 24/25 Java agent samples and 21/25 Python agent samples had no findings.
Human mean scores exceeded agent mean scores in both languages. Threshold tuning
cannot repair missing or inverted signal, so detector defaults remain unchanged.
A first Python-specific calibration pass extended `py-comment-depth` to detect
files containing at least three imperative comments that narrate routine
operations. On this fixed slice it added findings to 4 agent samples and 0 human
samples: Python agent zero-finding samples fell from 21/25 to 17/25 and the agent
mean rose from 1.50 to 2.62 per 100 lines, while the human distribution remained
unchanged. A second Python-specific pass added `py-demo-scaffolding`, which requires an
implementation definition, an explicit example/test heading, and subsequent
executable demonstration code outside designated example/demo/docs paths. It
added findings to 4 previously uncovered agent samples and 0 human samples:
Python agent zero-finding samples fell from 17/25 to 13/25 and the agent mean
rose from 2.62 to 4.12 per 100 lines, while the human distribution again
remained unchanged. A third pass extended `py-restating-comment` to require at
least three trailing comments that mechanically narrate inputs, parity branches,
or returns. It added a finding to 1 previously uncovered agent sample and 0 human
samples: Python agent zero-finding samples fell from 13/25 to 12/25 and the agent
mean rose from 4.12 to 4.59 per 100 lines, with no human-distribution change.
A fourth pass added `py-placeholder-scaffolding`, requiring at least two
independent unresolved template signals such as a `yourapp` import and an
"Assuming there's a function" comment. It added a finding to 1 previously
uncovered agent sample and 0 human samples: Python agent zero-finding samples
fell from 12/25 to 11/25 and the agent mean rose from 4.59 to 5.05 per 100 lines,
with the human distribution unchanged. This is calibration evidence, not
held-out validation; further detector work should use this split before
one-time evaluation on a disjoint test split.

A focused Java pass added a separate pre-LLM precision gate at
`evaluation/corpus/java-spring-human-precision-v1/`: 20 pinned files from five
established Spring projects. It then added `java-placeholder-scaffolding`, which
requires at least two comments leaving implementation assumptions or placeholder
logic unresolved. On the fixed AICD Java calibration cohort it added findings to
1 previously uncovered agent sample and 0 human samples. On the Spring
precision corpus it added no findings and no threshold-crossing false positives.
After strengthening import content checks to reject C# mislabeled as Java and
regenerating the deterministic slice, 22/25 Java agent samples have zero
findings; `java-placeholder-scaffolding` accounts for one covered agent sample. This narrow
result does not support broader Spring annotation or CRUD-boilerplate rules.

The TypeScript calibration source now applies the same contamination discipline:
it rejects C++, C#, PHP, Java, Go, and full HTML documents mislabeled as
TypeScript/JavaScript. Regeneration replaces the contaminated 13-agent slice
with a balanced 25 human / 25 agent cohort. Current slop rules produce 0 true
positives and 0 false positives on the cleaned cohort; 20/25 agent files and
22/25 human files have no findings. Earlier TypeScript recall figures from the
contaminated slice are invalid and must not guide detector work.

The generated `aicd-t2-calibration/` directory is intentionally untracked; the
pinned source specification and import lock make regeneration deterministic
without redistributing upstream samples.

A focused Go pass added `evaluation/corpus/go-human-precision-v1/`, containing
20 pre-LLM files from the Go project, Kubernetes, Prometheus, Cobra, and etcd.
All existing Go findings remain in the quality channel, but only 1/20 files is
finding-free and the mean quality score is `1.8800/100 lines`. This establishes
a needed precision-tightening gate. The first correction stops classifying the
idiomatic `_, err := call()` form as an ignored error, removing three false
attributions from the corpus. The pinned AICD source now requests its available
Go strata, but upstream HTTP 429 responses blocked regeneration during this
pass; no Go slop rule is justified until that agent cohort is materialized.

The current evidence requires a conservative interpretation of these reports:

- `exceeds_slop_threshold` means only that calibrated strong slop evidence crossed
  the configured threshold. It is not an authorship determination: human and
  agent-authored code can both be slop, and either can remain below the threshold.
- Every report includes explicit caveats and per-finding rule, file, line, and
  weight data so aggregate errors can be traced back to detector behavior.
- A pinned nine-sample AICD-Bench integration slice at `1.0/100 lines` produced
  1 true positive, 1 false positive, 4 true negatives, and 3 false negatives.
  This slice is too small and narrow for a population accuracy estimate, but it
  demonstrates that the current threshold is not a reliable general-purpose
  authorship classifier. Do not tune detector defaults to this integration
  slice; select operating points on a larger calibration split and report once
  on a disjoint held-out test split.

`mixed` and `unknown` samples are scored and reported but excluded from binary
metrics. A parse or unsupported-language failure remains visible on the sample
result and is counted in `excluded_errors`; it is never treated as a correct
human prediction or a false negative.

## TypeScript human precision corpus

[`corpus/typescript-human-precision-v1/`](corpus/typescript-human-precision-v1/)
contains 24 licensed files from immutable TypeScript, VS Code, Angular, NestJS,
Redux, and TypeORM revisions released by May 2021. Its first sweep exposed a
false positive in repeated declaration narration: `// Check if ...` comments
before `if` branches were being treated as declaration headings. Excluding
control flow reduced the corpus from one to zero slop findings while preserving
the two existing TypeScript calibration hits. The compatibility quality
aggregate remains noisy, led by nested ternaries and trivial wrappers; see the
corpus README for provenance and per-rule attribution.

## Dataset policy

A corpus used for release claims must satisfy all of the following:

1. **Immutable provenance.** Pin repository commits, dataset revisions, model
   identifiers, and generation settings. Do not use moving branches.
2. **Licensing.** Record a license for the corpus and every imported source.
   Do not check in code whose redistribution terms are unknown.
3. **No synthetic calibration.** Constructed fixtures belong in smoke tests,
   not calibration, validation, or reported test metrics.
4. **Split by origin.** Related files or patches from one repository must stay
   in one split to prevent repository-style leakage.
5. **Patch preservation.** For agent work, retain the base revision, patch,
   resulting files, prompt/session record when publishable, model version, and
   whether the sample is raw output or human-reviewed output.
6. **Human baseline hygiene.** Human labels require evidence predating agent
   contribution or a documented authorship source. “Not known to be AI” is
   `unknown`, not `human`.
7. **No threshold tuning on test.** Select thresholds using calibration data,
   iterate using validation data, and report the test split once per release.
8. **Report cohorts.** Aggregate metrics must be accompanied by language,
   generator family, sample unit, and raw-vs-reviewed cohorts. A single overall
   number can hide severe regressions.

## Initial scope and known limits

Version 1 evaluates the resulting source artifacts. It records patch and
repository sample units now so the manifest does not need to change when
patch-aware detectors arrive, but patch artifacts are not yet analyzed.
Likewise, this first slice reports one operating threshold; threshold sweeps,
confidence intervals, per-rule attribution, and baseline comparison belong in
the next iteration.

The immediate corpus-building target is Rust, TypeScript, Python, and Go, with
independent human, raw-agent, and reviewed-agent cohorts. Production patches
are the unit of interest; isolated contest solutions are not representative of
Lipstyk's intended use.

## Python human precision corpus

[`corpus/python-human-precision-v1/`](corpus/python-human-precision-v1/) contains
24 licensed files from immutable CPython, Flask, Click, Requests, pytest, and
NumPy revisions released by May 2021. It exists to measure false positives on
diverse, established human code. The legacy all-diagnostic aggregate crosses
`1.0/100 lines` for 12/24 files, but the separated slop channel produces
0/24 predictions; `py-trivial-wrapper` and `py-structural-repetition` account
for 58 of 78 quality diagnostics. See the corpus README for complete provenance,
attribution, and consequences.
