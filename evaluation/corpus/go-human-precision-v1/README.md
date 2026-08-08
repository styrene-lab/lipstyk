# Go human precision corpus v1

This corpus is a pre-LLM precision gate for Go rules. It covers the Go standard
library and toolchain, Kubernetes, Prometheus, Cobra, and etcd.

## Selection

The corpus contains 20 Go files: four manually stratified files from each of
five established projects. Every revision is pinned no later than June 2021.
The selection spans standard-library networking and encoding, command tooling,
Kubernetes machinery, observability, CLI infrastructure, distributed systems,
and integration support.

`import.lock.json` records each immutable repository revision, upstream path,
category, license, SHA-256 digest, and content-addressed local object. The corpus
is checked in so evaluation does not require network access.

This is a precision corpus, not an authorship benchmark. The `human` label means
that files come from established pre-LLM project history; it does not claim
independent verification of every line's author.

## Baseline

Run:

```bash
cargo run --bin lipstyk-eval -- \
  evaluation/corpus/go-human-precision-v1/corpus.json
```

All current Go findings use the quality channel, so 20/20 samples remain below
the slop threshold. The baseline nevertheless exposes substantial heuristic
noise. After the ignored-error correction, only 1/20 files has no quality findings and
the mean quality score is `1.8800/100 lines` (down from `1.9059`). The largest contributors are `go-restating-comment`,
`go-structural-repetition`, `go-antipattern`, and `go-error-handling`.

The first precision fix corrects the Go AST collector: `_, err := call()` no
longer counts as an ignored error. That syntax deliberately discards a value
while retaining the error and is idiomatic Go. The fix reduces false attribution
without changing rule thresholds or promoting quality findings to slop.

## Use

New Go slop rules must add coverage on a pinned agent calibration cohort without
introducing findings here. Existing quality rules should be tightened against
this corpus before their scores are interpreted as useful quality evidence.
Synthetic examples belong in unit tests, not this corpus.
