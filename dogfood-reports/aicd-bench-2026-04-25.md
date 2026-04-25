# AICD Bench Evaluation — 2026-04-25

lipstyk 0.1.0 evaluated against AICD Bench (arxiv 2602.02079), the
largest public benchmark for AI-generated code detection (2M samples,
77 generators, 9 languages).

## Task 1: Competitive Programming (binary classification)

200 samples per class. Short algorithmic solutions (6-45 lines).

| | AI | Human |
|---|---|---|
| Files with findings | 16/100 (16%) | 20/100 (20%) |
| Mean score | 0.26 | 0.29 |

**Result: No separation.** Competitive programming solutions have
identical structure regardless of origin — single functions, no
modules, no error handling, no documentation patterns. Lipstyk's
rules target production code patterns that don't exist here.

## Task 2: Production Code (model family attribution)

200 AI samples, 200 human samples. Real production code: Java, Python,
JavaScript, C++, Go, PHP. Multi-file, structured, with imports and
error handling.

| | AI (96 supported files) | Human (146 supported files) |
|---|---|---|
| Files with findings | 36 (38%) | 10 (7%) |
| Mean score | 1.09 | 0.12 |
| Total diagnostics | 74 | 12 |
| Documentation findings | 29 | 3 |
| Error-handling findings | 22 | 1 |

**Result: Clear separation on production code.**
- 5.4x more files flagged in AI set vs human set
- 9x higher mean score
- Documentation patterns are the strongest separator (9.7x)
- Error handling is the sharpest single signal (22x)

## What this means

Lipstyk detects AI-generated code patterns in production codebases,
not in algorithmic problem-solving. This is the intended use case —
it's built for catching slop in real projects, not for academic
binary classification on contest solutions.

The benchmark confirms what the research predicts: comment density
and error handling patterns are the strongest discriminators in
production code. Surface features (naming, whitespace) contribute
but are secondary.

## Limitations observed

- No Rust samples in our T2 pull (the benchmark has Rust but our
  random sample didn't include any). Rust-specific rules (the
  deepest part of lipstyk) were not exercised.
- Some files were unsupported language (Java, C#, PHP, C++) and
  skipped by lipstyk. The 96/200 and 146/200 file counts reflect
  language support gaps.
- T1 samples are too small and structurally uniform for lipstyk's
  density-based approach. This is expected, not a deficiency.

## Note

This evaluation was run with an early version of lipstyk (pre-oxc,
pre-Go, pre-DevOps). The rule set and parser infrastructure have
changed significantly since. Results should be treated as a baseline,
not current accuracy.

## Benchmark details

- Dataset: `AICD-bench/AICD-Bench` on HuggingFace
- Task 1: Robust Binary Classification (T1 split, test set)
- Task 2: Model Family Attribution (T2 split, test set)
- Paper: EACL 2026, Orel et al.
