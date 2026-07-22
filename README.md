# lipstyk

Static analysis for code-quality patterns that frequently appear in
machine-generated code. No ML, no authorship classifier — deterministic rules
you can read and argue with.

```
src/handler.rs — slop score: 42.5 (12.3/100 lines)
  src/handler.rs:4  [warn] comment restates the code: `create a new hashmap` (restating-comment)
  src/handler.rs:12 [warn] `.unwrap()` — consider propagating with `?` (unwrap-overuse)
  src/handler.rs:22 [SLOP] 4 step-by-step comments — AI loves narrating code like a tutorial (over-documentation)
  src/handler.rs:31 [warn] `fn process_data` — name is too vague to convey intent (generic-naming)
  src/handler.rs:31 [warn] `fn process_data` takes owned `String` — would `&str` work here? (string-params)
  src/handler.rs:45 [SLOP] `Err(_)` is silently swallowed or only logged (error-swallowing)

--- summary ---
files: 3/12 with findings
diagnostics: 23 (hint: 4, warn: 17, slop: 2)
total score: 42.5
elapsed: 8ms
```

## Why

AI-generated code compiles and runs but it reads like it was written
by someone who's never going to maintain it. `.unwrap()` everywhere.
`process_data` because that's what the training distribution picked.
Comments that narrate obvious code instead of explaining decisions.

Any single instance is fine. A file full of them is slop.

**If you're writing code with AI** — run it before you commit. Catch
the patterns your copilot leaves behind.

**If you're reviewing PRs** — `lipstyk --diff main` reports these patterns only
on changed lines. Use it as a focused review aid; do not treat a score as proof
of who or what authored the code.

**If you run infrastructure** — Dockerfiles running as root, K8s
manifests without resource limits, shell scripts without `set -e`,
CI workflows with hardcoded secrets. lipstyk catches the DevOps
patterns that AI gets wrong and humans miss in review.

**If you own a codebase** — track pattern density over time with JSON reports.
Set a threshold gate only after calibrating it against accepted and rejected
changes in that codebase; the threshold is a policy limit, not a universal
classifier cutoff.

## Install

```bash
cargo install --git https://github.com/styrene-lab/lipstyk
```

Or from source:

```bash
git clone https://github.com/styrene-lab/lipstyk
cd lipstyk
cargo build --release
cp target/release/lipstyk ~/.local/bin/
```

## Usage

```bash
lipstyk src/                                          # analyze everything
lipstyk --exclude-tests src/                          # skip #[test] / #[cfg(test)]
lipstyk --diff main --exclude-tests src/              # only changed lines
lipstyk --exclude-tests --threshold 20 src/           # CI gate
```

### Output formats

```bash
lipstyk src/                          # terminal (default)
lipstyk --json src/                   # full JSON report
lipstyk --sarif src/                  # SARIF 2.1.0 for GitHub code scanning
lipstyk --report src/                 # Markdown for PR comments
lipstyk --summary src/                # one line per file
```

## Languages

| Language | Extensions | Rules | Analysis |
|----------|-----------|-------|----------|
| Rust | `.rs` | 21 | AST via `syn` |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | 14 | AST via `oxc` + text |
| Python | `.py` | 15 | AST via `tree-sitter` + text |
| Go | `.go` | 8 | AST via `tree-sitter` + text |
| HTML / CSS | `.html` `.htm` `.css` `.vue` `.svelte` | 6 | tag parser |
| Java | `.java` | 4 | text (legacy) |
| Elixir | `.ex` `.exs` | 5 | text |
| Shell | `.sh` `.bash` `.zsh` | 3 | text |
| Dockerfile | `Dockerfile` `Containerfile` | 1 | text (5 checks) |
| Kubernetes YAML | `.yml` `.yaml` | 1 | content-sniffed (6 checks) |
| CI/CD YAML | `.yml` `.yaml` | 1 | content-sniffed (5 checks) |
| Markdown | `.md` `.mdx` | 3 | text |

82 rules. Full reference in [RULES.md](RULES.md).

## What it catches

Rust: `.unwrap()` chains, gratuitous `.clone()`, `Box<dyn Error>`
catch-alls, verbose match arms, C-style index loops, needless type
annotations and lifetimes, `String` params where `&str` works

TS/JS (oxc AST): `any` abuse, empty/log-only catch blocks,
`async` without `await`, `console.log` dumps, nested ternaries,
Promise anti-patterns, structural repetition, deep nesting

Go: bare `return err`, `interface{}` overuse, `panic()` in library
code, `fmt.Println` debugging, `time.Sleep` sync, structural
repetition via tree-sitter AST

Python: bare `except:`, `print()` debugging, `from X import *`,
mutable default arguments, `range(len(x))` loops, type hint gaps

Comments (all languages): restating code, step-by-step narration,
per-function density, uniform spacing

Naming: `process_data`, `handle_request`, `fetchData`, vague TODOs,
naming entropy

DevOps: Dockerfiles running as root, K8s without resource limits,
wildcard RBAC, hardcoded CI secrets, shell scripts without `set -e`

Markdown: AI buzzword density, placeholder content, template structure

Cross-file: duplicate blocks, identical imports, cloned error handling

## Configuration

`.lipstyk.toml` in the project root:

```toml
[settings]
exclude_tests = true
threshold = 20

[rules.redundant-clone]
weight = 0.25         # downweight for Axum/actix projects

[rules.structural-repetition]
enabled = false
```

Auto-discovered by walking parent directories.

## CI

```yaml
# gate
- run: lipstyk --exclude-tests --threshold 20 src/

# diff-only PR check
- run: lipstyk --diff origin/${{ github.base_ref }} --exclude-tests --threshold 15 src/

# SARIF for inline annotations
- run: lipstyk --sarif --exclude-tests src/ > lipstyk.sarif
  continue-on-error: true
- uses: github/codeql-action/upload-sarif@v3
  if: always()
  with:
    sarif_file: lipstyk.sarif
  continue-on-error: true

# markdown in job summary
- run: lipstyk --report --exclude-tests src/ >> $GITHUB_STEP_SUMMARY
  continue-on-error: true
```

## LSP server

Inline diagnostics in any editor that speaks LSP.

```bash
cargo build --release --features lsp
```

Configure your editor to use `lipstyk-lsp` as a language server.
See [INTEGRATION.md](INTEGRATION.md) for VS Code, Neovim, and
Helix setup.

## pre-commit

The hook is self-contained: pre-commit builds the pinned Lipstyk revision with
Cargo, so contributors do not need to install `lipstyk` separately.

```yaml
repos:
  - repo: https://github.com/styrene-lab/lipstyk
    rev: v0.2.1                 # pin a release tag or immutable commit SHA
    hooks:
      - id: lipstyk             # scans the staged source files
```

For a lower-noise changed-line gate, use `id: lipstyk-diff` instead. Do not use
`rev: main` in a real project: moving revisions make hook environments
non-reproducible.

## Agent integration

The `lipstyk-agent` binary speaks Omegon RPC and MCP (`--mcp` flag).

```bash
cargo build --release --features agent
```

Register as an MCP server (Claude Code, Cursor, VS Code, Zed):

```json
{
  "mcpServers": {
    "lipstyk": {
      "command": "lipstyk-agent",
      "args": ["--mcp"]
    }
  }
}
```

Tools: `lipstyk_check` (self-review with fix suggestions),
`lipstyk_diff` (changed lines only), `lipstyk_report` (markdown),
`lipstyk_rules` (list all rules).

CLAUDE.md snippet for automatic self-review:

```markdown
After modifying code, call lipstyk_check on the changed file. If
verdict is "suspicious" or "sloppy", fix the top findings and
re-check. Don't commit until pass: true.
```

Full integration guide for Omegon, Cursor, Windsurf, Cline, Aider,
and CI in [INTEGRATION.md](INTEGRATION.md).

## Dogfooding

Lipstyk analyzes itself. Reports in
[`dogfood-reports/`](dogfood-reports/).

Current self-scan: score 29.0, 29/108 files with findings, mostly hints.

## Scoring

Diagnostics carry weights (0.1-3.0). File score = sum of weights.
`score_per_100_lines` normalizes for size.

Rules escalate by count: one `.clone()` is a 0.5 hint; fifteen+
escalates to warning. Single findings don't mean much. Density does.

Verdicts: clean (<5), mild (<15), suspicious (<30), sloppy (>=30).

## Research

Rule design draws from published work on patterns associated with
machine-generated code, but Lipstyk has not established a general-purpose
authorship-classification accuracy claim. Current evidence supports using the
rules as review signals:

- Comment density can be informative in aggregate, but is not a reliable
  standalone authorship discriminator.
- Function-level structure, naming entropy, and cross-file uniformity are useful
  code-quality signals whose authorship meaning depends on language, model,
  prompting, editing, and repository context.
- Model and workflow drift means thresholds require periodic recalibration.
- A pinned nine-sample AICD-Bench integration slice found 1/4 agent samples and
  falsely flagged 1/5 human samples at `1.0/100 lines`. The slice is too small
  for a population estimate, but it falsifies any assumption that the current
  score is a broadly reliable authorship classifier.

Citations in [RULES.md](RULES.md). The evaluation harness and reproducible
external-corpus importer live under [`evaluation/`](evaluation/README.md).
Reported metrics describe only the named corpus revision and threshold.

## Limitations

Detects patterns, not authorship or intent. Scores are weighted code-quality
signals, not probabilities. Expect false positives on human code and false
negatives on generated or edited output; performance varies by language and
corpus. Calibrate CI thresholds locally and read the underlying findings.

## License

MIT
