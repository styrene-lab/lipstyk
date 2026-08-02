# Java/Spring human precision corpus v1

This corpus is a pre-LLM precision gate for Java rules, with deliberate coverage
of Spring Framework, Spring Boot, Spring Security, Spring Data JPA, and Spring
REST Docs code.

## Selection

The corpus contains 20 Java files: four manually stratified production or
framework-support files from each of five established Spring projects. Every
source revision predates June 2021. The selection covers core framework code,
web/MVC and controller code, configuration, persistence, security, test support,
and documentation infrastructure.

`import.lock.json` records the repository, immutable commit, upstream path,
category, license, SHA-256 digest, and local object name for every sample. Object
filenames are content-addressed. The corpus is checked in so evaluation does not
require network access.

This is a precision corpus, not an authorship benchmark. Its `human` labels mean
that the pinned revisions are established pre-LLM project history; they do not
imply that every line's individual author was independently verified.

## Baseline

Run:

```bash
cargo run --bin lipstyk-eval -- \
  evaluation/corpus/java-spring-human-precision-v1/corpus.json
```

Before `java-placeholder-scaffolding`, the default rules produced no
threshold-crossing false positives: 20/20 samples remained below the configured
`1.0/100 lines` threshold. Eighteen had zero scores. Two files had low-level
findings from existing Java rules, with a mean score of `0.1744/100 lines`.

The new placeholder rule also leaves all 20 files unchanged. This supports the
rule's narrow requirement for at least two unresolved placeholder comments in a
single file. It does not establish recall or justify broader Spring annotation,
CRUD-boilerplate, or framework-layer heuristics.

## Use

New Java and Spring-specific rules must pass this corpus without increasing the
threshold-crossing false-positive count. Candidate rules should also improve
coverage on a pinned agent calibration cohort before they are enabled by
default. Synthetic examples remain unit-test fixtures and must not be added to
this corpus.
