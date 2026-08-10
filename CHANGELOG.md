# Changelog

## [0.3.0](https://github.com/styrene-lab/lipstyk/compare/v0.2.1...v0.3.0) - 2026-08-14

### Added

- Detect Python demo and placeholder scaffolding, imperative narration, repeated inline narration, and broad `except: pass` patterns.
- Detect TypeScript placeholder scaffolding, repeated declaration narration, redundant async functions, and polished disclaimer text.
- Detect unresolved Java placeholder scaffolding.
- Add versioned evaluation manifests, deterministic imports, score distributions, diff-mode evaluation, and pre-LLM precision corpora for TypeScript, Java/Spring, and Go.
- Import revision-pinned Hugging Face Parquet shards directly with a local cache.

### Changed

- Separate quality diagnostics from machine-generation scoring so generic quality findings do not inflate the slop score.
- Tighten calibration imports to reject mislabeled PHP, C#, C++, Java, Go, and HTML rows.
- Improve Elixir bang-call detection, including idiomatic allowlisting and UTF-8-safe matching.
- Upgrade first-party GitHub Actions to Node.js 24-compatible releases.

### Fixed

- Handle the agent `initialize` RPC method.
- Preserve retained Go errors in idiomatic blank-result assignments such as `_, err := call()`.
- Correct release binary path resolution in CI.

## [0.2.1](https://github.com/styrene-lab/lipstyk/compare/v0.2.0...v0.2.1) - 2026-05-16

### Added

- Add release automation and cross-platform binary packaging.

### Fixed

- Restore Linux ARM64 release builds without an OpenSSL dependency.
