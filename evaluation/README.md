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
100 AICD-Bench T2 training samples: 25 human and 25 agent samples for each of
Python and Java. Rebuild it with:

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
This is calibration evidence, not held-out validation; further detector work
should use this split before one-time evaluation on a disjoint test split.

The generated `aicd-t2-calibration/` directory is intentionally untracked; the
pinned source specification and import lock make regeneration deterministic
without redistributing upstream samples.

The current evidence requires a conservative interpretation of these reports:

- `predicted_agent` is retained as a version-1 compatibility field. It means
  only that the configured pattern-score threshold was crossed; it is **not**
  a verified authorship determination or probability.
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
