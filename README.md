# lipstyk

Anti-AI-slop code analysis. Detects machine-generated code patterns
through static analysis — no ML classifiers, no language models.
Every finding is a deterministic rule you can read, understand, and
disagree with.

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

AI code generation produces recognizable patterns. Not because the
code is wrong — it usually compiles and runs — but because it's
*generated without intent*. The same `.unwrap()` everywhere instead
of error propagation. The same `process_data` naming from the
training distribution. Comments that restate the code instead of
explaining why.

Lipstyk catches these patterns so you can fix them before they
compound. It's designed for agentic development workflows where AI
writes most of the code and you need a quality gate that's faster
than manual review.

**The core principle:** any single finding is inconclusive. Density
is the signal.

## Install

```bash
cargo install --git https://github.com/styrene-lab/lipstyk
```

Or build from source:

```bash
git clone https://github.com/styrene-lab/lipstyk
cd lipstyk
cargo build --release
cp target/release/lipstyk ~/.local/bin/
```

## Usage

```bash
# Analyze files or directories (auto-detects language)
lipstyk src/
lipstyk src/handler.rs src/lib.rs

# Exclude test code (recommended for Rust)
lipstyk --exclude-tests src/

# Only score lines changed since main
lipstyk --diff main --exclude-tests src/

# CI gate — exit 0 unless any file exceeds score 20
lipstyk --exclude-tests --threshold 20 src/
```

### Output Formats

```bash
lipstyk src/                          # human-readable (default)
lipstyk --json src/                   # full JSON report
lipstyk --sarif src/                  # SARIF 2.1.0 (GitHub code scanning)
lipstyk --report src/                 # Markdown (PR comments, docs)
lipstyk --summary src/                # one line per file
```

## Languages

| Language | Extensions | Rules | Analysis |
|----------|-----------|-------|----------|
| Rust | `.rs` | 21 | AST-level via `syn` |
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | 7 | Text-based |
| Python | `.py` | 7 | Text-based |
| HTML / CSS | `.html` `.htm` `.css` `.vue` `.svelte` | 6 | Tag-aware parser |

41 rules total. See [RULES.md](RULES.md) for the full reference with
descriptions, severity levels, and research basis.

## Rules at a Glance

**Error handling** — `.unwrap()` overuse, silently swallowed errors,
`Box<dyn Error>` catch-all, bare `except:` in Python

**Ownership** (Rust) — gratuitous `.clone()`, owned `String` params
where `&str` works, needless lifetime annotations

**Documentation** — comments that restate code ("// increment counter"
above `counter += 1`), step-by-step tutorial narration, mechanically
uniform comment spacing

**Naming** — `process_data`, `handle_request`, `fetchData`, generic
TODOs ("TODO: add error handling")

**Structure** — trivial wrapper functions, everything `pub`, 6+ derives
stacked on one type, `#[allow(dead_code)]` instead of deleting code

**Statistical** — blank line regularity, line length uniformity,
function shape repetition, naming entropy

**HTML/CSS** — div soup, missing semantic elements, inline styles,
generic class names, accessibility gaps, `!important` overuse

**TS/JS** — `any` abuse, `console.log` dumps, nested ternaries,
Promise anti-patterns

**Python** — bare `except:`, `print()` debugging, `from X import *`,
inconsistent type hints

## Configuration

Create `.lipstyk.toml` in your project root:

```toml
[settings]
exclude_tests = true
threshold = 20

[rules.redundant-clone]
weight = 0.25         # downweight for Axum/actix projects

[rules.structural-repetition]
enabled = false       # disable entirely
```

Rules not listed use defaults. Config is auto-discovered by walking
parent directories.

## CI / GitHub Actions

### Quality gate

```yaml
- name: Slop check
  run: lipstyk --exclude-tests --threshold 20 src/
```

### Diff-only PR check

```yaml
- name: Slop check (changed lines only)
  run: lipstyk --diff origin/${{ github.base_ref }} --exclude-tests --threshold 15 src/
```

### SARIF upload (inline PR annotations)

```yaml
- name: Lipstyk analysis
  run: lipstyk --sarif --exclude-tests src/ > lipstyk.sarif
  continue-on-error: true

- name: Upload SARIF
  if: always()
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: lipstyk.sarif
  continue-on-error: true
```

### Markdown report in job summary

```yaml
- name: Lipstyk report
  run: lipstyk --report --exclude-tests src/ >> $GITHUB_STEP_SUMMARY
  continue-on-error: true
```

## Agent Integration

Lipstyk includes an agent binary (`lipstyk-agent`) that speaks both
the Omegon extension protocol and MCP. One binary, two protocols.

```bash
cargo build --release --features agent
```

### Claude Code / Cursor / VS Code / Zed

Add to `.mcp.json` in your project root:

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

### Agent tools

| Tool | Purpose |
|------|---------|
| `lipstyk_check` | Self-review with pass/fail verdict and fix suggestions |
| `lipstyk_diff` | Score only changed lines since a git ref |
| `lipstyk_report` | Generate Markdown report for PR comments or docs |
| `lipstyk_rules` | List all rules with categories and weights |

### CLAUDE.md snippet

```markdown
After modifying code, call lipstyk_check with `path` set to the file
you changed. If verdict is "suspicious" or "sloppy", fix the
highest-severity findings and re-check. Do not commit until
lipstyk_check returns pass: true.
```

See [INTEGRATION.md](INTEGRATION.md) for detailed setup for Omegon,
Cursor, Windsurf, Cline, Aider, and CI pipelines.

## Dogfooding

Lipstyk analyzes itself. Reports are in
[`dogfood-reports/`](dogfood-reports/) — anyone can see what the
tool flags on its own code.

Current self-scan: **score 20.3, 0.4/100 lines, verdict mild.**

## Scoring

Each diagnostic has a weight (0.1–3.0). A file's slop score is the
sum of all weights. The `score_per_100_lines` metric normalizes for
file size.

Many rules escalate severity by count: one `.clone()` is a hint
(0.5); ten in one file is slop (2.0). This reflects the core
principle — any single pattern is inconclusive; density is the
signal.

**Verdicts:** clean (<5), mild (<15), suspicious (<30), sloppy (≥30).

## Research Basis

The rule set is informed by academic research on detecting
machine-generated code:

- Comment-to-code ratio is the universal discriminator across every
  multi-language study (CoDet-M4, SANER 2025 multilingual stylometry)
- Function-level analysis is 8.6x more discriminative than file-level
- AI distributes comments uniformly; humans cluster around complexity
- Naming diversity separates human from AI code
- Newer models are harder to detect — rule-based detection has a
  shelf life, and transparency is a feature

See the Research Basis section in [RULES.md](RULES.md) for citations.

## Limitations

This tool detects patterns, not intent. It will produce:

- **False positives** on human code that happens to be verbose,
  uniformly formatted, or heavily commented
- **False negatives** on AI code that has been manually edited or
  generated by models trained to avoid these patterns

It is not a substitute for code review. It is a signal that something
warrants closer inspection.

## License

MIT
