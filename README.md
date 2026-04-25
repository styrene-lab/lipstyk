# lipstyk

Static analysis for machine-generated code patterns. No ML, no
classifiers — deterministic rules you can read and argue with.

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
| TypeScript / JavaScript | `.ts` `.tsx` `.js` `.jsx` | 7 | text |
| Python | `.py` | 7 | text |
| HTML / CSS | `.html` `.htm` `.css` `.vue` `.svelte` | 6 | tag parser |

41 rules. Full reference in [RULES.md](RULES.md).

## What it catches

Rust: `.unwrap()` chains, gratuitous `.clone()`, `Box<dyn Error>`
catch-alls, verbose match arms, C-style index loops, needless type
annotations and lifetimes, `String` params where `&str` works

Naming: `process_data`, `handle_request`, `fetchData`, vague TODOs,
low naming entropy across a file

Comments: restating what the code says, step-by-step tutorial
narration, mechanically uniform comment spacing, high per-function
comment density

Structure: trivial wrapper clusters, everything `pub`, derive
stacking, `#[allow(dead_code)]` papering over unused code, functions
with identical AST shapes

Statistical: blank line regularity, line length uniformity

HTML/CSS: div soup, missing semantic elements, inline styles, generic
class names, accessibility gaps, `!important` abuse, magic pixel
values

TS/JS: `any` everywhere, `console.log` left in, nested ternaries,
`.then().catch(() => {})` chains

Python: bare `except:`, `print()` debugging, `from X import *`,
inconsistent type hints

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

Current self-scan: score 20.3, 0.4 per 100 lines, mild.

## Scoring

Diagnostics carry weights (0.1-3.0). File score = sum of weights.
`score_per_100_lines` normalizes for size.

Rules escalate by count: one `.clone()` is a 0.5 hint; ten in the
same file is a 2.0 slop. Single findings don't mean much. Density
does.

Verdicts: clean (<5), mild (<15), suspicious (<30), sloppy (>=30).

## Research

Rule design draws from:

- CoDet-M4 (ACL 2025) and SANER 2025 multilingual stylometry:
  comment-to-code ratio as universal discriminator
- Function-level granularity is 8.6x more discriminative than
  file-level
- AI distributes comments uniformly; humans cluster near complexity
- Naming entropy separates human and AI code
- Detection accuracy drops with newer models (0.96 AUC for GPT-3.5,
  0.68 for Claude 3 Haiku) — the rules will need to evolve

Citations in [RULES.md](RULES.md).

## Limitations

Detects patterns, not intent. False positives on verbose human code.
False negatives on edited AI output. Not a replacement for reading
the code.

## License

MIT
