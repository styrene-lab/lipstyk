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
