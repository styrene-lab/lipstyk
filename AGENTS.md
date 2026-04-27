# AGENTS.md

lipstyk is a static analysis tool that detects machine-generated code
patterns. Deterministic rules, no ML. Rust codebase, edition 2024.

## Build

```bash
cargo build --release                    # CLI only
cargo build --release --features agent   # CLI + MCP/Omegon agent
cargo build --release --features lsp     # CLI + LSP server
cargo test                               # 87 integration tests
```

CI runs `cargo clippy -- -D warnings`. Fix all warnings before opening
a PR.

## Architecture

```
src/
  rules/          # Rust-specific rules (AST via syn)
  ts/             # TypeScript/JavaScript rules (AST via oxc + text)
  python/         # Python rules (AST via tree-sitter + text)
  golang/         # Go rules (AST via tree-sitter + text)
  html/           # HTML/CSS rules (tag parser)
  java/           # Java rules (text)
  shell/          # Shell rules (text)
  docker/         # Dockerfile rules (text)
  devops/         # Kubernetes + CI/CD YAML rules (content-sniffed)
  markdown/       # Markdown rules (text)
  common/         # Shared analysis utilities
  lint.rs         # Linter core — rule registration and dispatch
  source_rule.rs  # SourceRule trait, Lang enum, SourceContext
  diagnostic.rs   # Diagnostic, Severity, SlopScore
  config.rs       # .lipstyk.toml loading
  diff.rs         # --diff mode (changed-lines filtering)
  report.rs       # Output formatting (JSON, SARIF, Markdown, summary)
  main.rs         # CLI entry point
  agent.rs        # MCP/Omegon entry point (behind `agent` feature)
  lsp.rs          # LSP server (behind `lsp` feature)
```

## Adding a rule

### 1. Pick the right trait

**Rust rules** implement `Rule` (uses `syn` AST):

```rust
pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic>;
}
```

`LintContext` carries `filename`, `source`, and `exclude_tests`.

**All other languages** implement `SourceRule`:

```rust
pub trait SourceRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn langs(&self) -> &[Lang];
    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic>;
}
```

`SourceContext` carries `filename`, `source`, `lang`, and optional
pre-parsed data: `html` (for HTML/CSS), `oxc` (for TS/JS), `go` (for
Go). The dispatch only calls your rule when the file's language matches
one of your `langs()`, so pre-parsed fields are always `Some` when
your rule needs them.

### 2. Write the rule

Create a file in the appropriate language directory. Name it after the
rule with underscores (e.g., `src/rules/my_new_rule.rs`).

Return a `Vec<Diagnostic>`:

```rust
Diagnostic {
    rule: "my-new-rule",       // kebab-case, matches name()
    message: String,           // human-readable, concise
    line: usize,               // 1-indexed line number
    severity: Severity::Hint,  // Hint, Warning, or Slop
    weight: 0.5,               // score contribution (0.1-3.0)
}
```

Severity/weight guidelines:
- **Hint** (0.1-0.5): might be human, suspicious in aggregate
- **Warning** (0.5-1.5): likely slop pattern
- **Slop** (1.5-3.0): strong machine-generation indicator

Escalate by count within a file. One occurrence = Hint. Many = Warning
or Slop. Density is the signal, not presence.

### 3. Register it

Add the module to the appropriate `mod.rs`, then register in
`src/lint.rs` inside `Linter::with_defaults()`:

```rust
// Rust rule
linter.add_rust_rule(Box::new(crate::rules::my_new_rule::MyNewRule));

// Source rule
linter.add_source_rule(Box::new(crate::ts::my_new_rule::MyNewRule));
```

### 4. Test it

Tests live in `tests/rules.rs`. Use the helpers:

```rust
#[test]
fn my_new_rule_fires() {
    assert!(has_rule(r#"fn bad() { /* sloppy code */ }"#, "t.rs", "my-new-rule"));
}

#[test]
fn my_new_rule_clean() {
    assert!(no_rule(r#"fn good() { /* clean code */ }"#, "t.rs", "my-new-rule"));
}
```

Write at least one positive case (fires) and one negative case (clean
code doesn't trigger). Test edge cases: threshold boundaries, test
code exclusion, language-specific quirks.

For multi-language rules, test each language variant with the
appropriate file extension (`"t.ts"`, `"t.py"`, `"t.go"`).

### 5. Document it

Add an entry to `RULES.md` under the appropriate language section.

## Configuration

Rules can be disabled or reweighted via `.lipstyk.toml`:

```toml
[rules.my-new-rule]
enabled = false

[rules.my-new-rule]
weight = 0.25
```

Config is discovered by walking parent directories. Your rule doesn't
need to handle this — the linter filters disabled rules and applies
weight overrides automatically.

## Conventions

- Rule names are kebab-case: `unwrap-overuse`, `bare-except`
- One rule per file, struct name matches the concept
- Rules must be deterministic — no randomness, no network calls
- Prefer AST analysis over text matching when a parser is available
- Don't add rules that duplicate what clippy/eslint/ruff already catch
  well. Focus on patterns characteristic of machine generation
- Keep diagnostics actionable: say what to do, not just what's wrong
- Test code (`#[test]`, `#[cfg(test)]`) should be downweighted or
  skipped — unwraps in tests are fine

## Dogfooding

lipstyk scans itself in CI. Reports go to `dogfood-reports/`. The
threshold is 60 for the full scan and 15 for diff-only PR checks. If
your change raises the score past the threshold, either fix the
findings or adjust the config with justification.
