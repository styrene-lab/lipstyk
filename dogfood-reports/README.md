# Dogfood Reports

Lipstyk analyzes itself on every significant change. Reports are
committed to the repo for transparency — anyone can see what the
tool flags on its own code and what it misses.

## Current Scorecard

| Date | Score | Files | /100L | Verdict | Top Category |
|------|-------|-------|-------|---------|-------------|
| 2026-04-25 | [20.3](self-2026-04-25.md) | 17/62 | 0.4% | Mild | naming |

## How to Read These

**Score** — sum of all diagnostic weights across all files.

**/100L** — score normalized per 100 lines of code. This is the
density metric that's comparable across projects and over time.
Under 1.0 is clean; 1-3 is mild; 3+ warrants investigation.

**Verdict** — clean (<5), mild (<15), suspicious (<30), sloppy (>=30).

## Generating Reports

```bash
lipstyk --report --exclude-tests src/ > dogfood-reports/self-$(date +%Y-%m-%d).md
lipstyk --json --exclude-tests src/ > dogfood-reports/self-$(date +%Y-%m-%d).json
```

## Report Format

Each report includes:
- Markdown (`.md`) — human-readable, renderable on GitHub/Forgejo
- JSON (`.json`) — machine-readable, same schema as `--json` output

The JSON reports are the source of truth for trend tracking.
