# Lipstyk Agent Integration Guide

Lipstyk exposes three tools to agent harnesses:

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `lipstyk_check` | Self-review with pass/fail and fix suggestions | After writing/modifying code, before committing |
| `lipstyk_diff` | Score only changed lines since a git ref | PR review, pre-commit gate |
| `lipstyk_rules` | List all rules across all languages | Discovery, configuration |

The agent binary (`lipstyk-agent`) speaks both Omegon RPC and MCP
natively via the `--mcp` flag. One binary, two protocols.

---

## Build & Install

```bash
# Build
cargo build --release --features agent

# Install the CLI and agent binaries
cp target/release/lipstyk target/release/lipstyk-agent ~/.local/bin/

# Or symlink for development
ln -s $(pwd)/target/release/lipstyk ~/.local/bin/lipstyk
ln -s $(pwd)/target/release/lipstyk-agent ~/.local/bin/lipstyk-agent
```

Ensure `~/.local/bin` is in your `$PATH`.

---

## Omegon

Lipstyk is a native Omegon extension.

### Install

```bash
# Development — symlink the repo
ln -s /path/to/lipstyk ~/.omegon/extensions/lipstyk

# Build the agent binary first — omegon expects it at the path in manifest.toml
cd /path/to/lipstyk && cargo build --release --features agent
```

The `manifest.toml` at the repo root handles registration. Tools
appear on next `omegon restart`. If tools don't appear, check
`omegon extension list` for errors, and ensure the binary at
`target/release/lipstyk-agent` exists.

### Posture integration

Add to your posture's tool policy:

```toml
[tools.lipstyk_check]
auto_approve = true    # Read-only analysis — no confirmation needed

[tools.lipstyk_diff]
auto_approve = true
```

### Delegation prompt

```
Before committing, run lipstyk_diff with path set to the working
directory and base set to "HEAD". If verdict is "suspicious" or
"sloppy", address the highest-severity findings and re-check.
```

---

## Claude Code

Lipstyk integrates with Claude Code as an MCP server via `--mcp`.

### Project-level (recommended)

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

### CLAUDE.md integration

Add to your project's `CLAUDE.md` for automatic self-review:

```markdown
## Code quality

After modifying code, call the lipstyk_check tool with `path` set
to the file you changed. If verdict is "suspicious" or "sloppy",
fix the highest-severity findings and re-check. Do not commit until
lipstyk_check returns pass: true.

For pre-commit review of all changes, call lipstyk_diff with
`path` set to the project root and `base` set to "HEAD".
```

### Debugging

If tools don't appear, set the log environment variable:

```json
{
  "mcpServers": {
    "lipstyk": {
      "command": "lipstyk-agent",
      "args": ["--mcp"],
      "env": {
        "LIPSTYK_LOG": "lipstyk=debug"
      }
    }
  }
}
```

Diagnostic output goes to stderr (not mixed into the MCP protocol).

---

## Cursor

Add to `.cursor/mcp.json` in your project root:

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

---

## Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

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

---

## Zed

Add to Zed's MCP configuration (Settings > Extensions > MCP):

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

---

## VS Code

Add to `.vscode/mcp.json` in your project root:

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

---

## Cline

Add to `~/.cline/mcp_settings.json`:

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

---

## Aider

Aider does not support MCP, but can use lipstyk as a lint command:

```bash
aider --lint-cmd "lipstyk --exclude-tests --threshold 20"
```

This runs lipstyk after each edit. Non-zero exit triggers a fix cycle.

---

## CI / GitHub Actions

The standalone CLI works without any agent harness.

### Basic quality gate

```yaml
- name: Lipstyk slop check
  run: lipstyk --exclude-tests --threshold 20 src/
```

**Exit codes:**
- `0` — No file exceeded the threshold (or no findings with `--threshold`)
- `1` — At least one file exceeded the threshold, or any findings exist
  if no threshold is set

### Diff-only mode (PR check)

```yaml
- name: Lipstyk diff check
  run: lipstyk --diff origin/${{ github.base_ref }} --exclude-tests --threshold 15 src/
```

Only scores lines changed in the PR. Combine with `--threshold` for
a hard gate that doesn't penalize existing code.

### SARIF upload (GitHub code scanning)

```yaml
- name: Lipstyk analysis
  run: lipstyk --sarif --exclude-tests src/ > lipstyk.sarif
  continue-on-error: true

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  if: always()
  with:
    sarif_file: lipstyk.sarif
```

Creates inline annotations on PRs. `continue-on-error` is needed
because lipstyk returns exit 1 when findings exist, and the SARIF
upload step needs to run regardless.

### JSON report for dashboards

```yaml
- name: Lipstyk report
  run: lipstyk --json --exclude-tests src/ > lipstyk-report.json
  continue-on-error: true

- name: Upload artifact
  uses: actions/upload-artifact@v4
  with:
    name: lipstyk-report
    path: lipstyk-report.json
```

The JSON report includes per-file scores, per-rule breakdowns,
per-category aggregates, git metadata, and duration — ready for
Grafana, VictoriaMetrics, or any dashboard that consumes JSON.

---

## Tool Reference

### `lipstyk_check`

Self-review tool. Returns a compact, agent-optimized response.

**Parameters:**
- `code` (string) — Source code to check
- `filename` (string) — Filename for language detection (e.g. `app.tsx`)
- `path` (string) — File or directory to check from disk

Provide either `code` + `filename` or `path`.

**Response:**
```json
{
  "pass": true,
  "verdict": "mild",
  "score": 9.5,
  "files_scanned": 1,
  "files_with_findings": 1,
  "total_findings": 7,
  "by_severity": { "slop": 0, "warning": 6, "hint": 1 },
  "categories": [
    { "category": "naming", "count": 3, "weight": 4.5 }
  ],
  "findings": [
    {
      "file": "handler.rs",
      "line": 1,
      "rule": "generic-naming",
      "severity": "Warning",
      "message": "`fn process_data` — name is too vague",
      "fix": "Rename to describe what this specifically does"
    }
  ]
}
```

**Verdicts:**
- `clean` (< 5) — No action needed
- `mild` (< 15) — Acceptable, review hints if you want
- `suspicious` (< 30) — Address warnings before committing
- `sloppy` (≥ 30) — Significant rework needed

### `lipstyk_diff`

Diff-aware analysis. Same response format as `lipstyk_check`.

**Parameters:**
- `path` (string, required) — File or directory to check
- `base` (string) — Git ref to diff against. Examples: `HEAD`, `main`,
  `origin/main`. Default: unstaged changes.

### `lipstyk_rules`

Returns metadata about supported languages and rule counts.

---

## Configuration

Place a `.lipstyk.toml` in your project root:

```toml
[settings]
exclude_tests = true     # Suppress test-code findings
threshold = 20           # CI gate threshold

[rules.structural-repetition]
enabled = false          # Disable this rule entirely

[rules.pub-overuse]
weight = 0.5             # Reduce this rule's weight

[rules.restating-comment]
enabled = false          # Your team prefers verbose comments
```

The config is auto-discovered by walking parent directories.
Both the CLI and the agent extension respect it.

---

## Troubleshooting

**Tools don't appear in Claude Code / Cursor:**
- Verify the binary is on `$PATH`: `which lipstyk-agent`
- Test manually: `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | lipstyk-agent --mcp`
- Check stderr for errors: `lipstyk-agent --mcp 2>/tmp/lipstyk-debug.log`

**MCP server crashes on startup:**
- Ensure you built with `--features agent`: `cargo build --release --features agent`
- The `lipstyk` binary (without `-agent`) is the CLI, not the agent. They are separate binaries.

**Agent reports "tool not found":**
- Restart the editor/agent harness after adding the MCP config
- Check that `.mcp.json` is valid JSON (trailing commas will break it)

**Too many false positives:**
- Add a `.lipstyk.toml` to your project root to tune or disable specific rules
- Use `--exclude-tests` or set `exclude_tests = true` in config
- Use `--diff` to only check changed lines

**Debugging agent protocol:**
- Set `LIPSTYK_LOG=lipstyk=debug` in the MCP server's `env` config
- All diagnostic output goes to stderr, never stdout (stdout is the protocol channel)

---

## Supported Languages

| Language | Extensions | Rules |
|----------|-----------|-------|
| Rust | `.rs` | 21 (AST-level via `syn`) |
| TypeScript/JavaScript | `.ts`, `.tsx`, `.js`, `.jsx` | 7 |
| Python | `.py` | 7 |
| HTML/CSS | `.html`, `.htm`, `.css`, `.vue`, `.svelte` | 6 |
