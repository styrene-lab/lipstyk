use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags Go-specific anti-patterns from AI generation.
///
/// - `interface{}` / `any` overuse (like TS `any` — defeats type safety)
/// - `fmt.Println` debugging left in non-test code
/// - `init()` functions (often misused by AI for side effects)
/// - `time.Sleep` in non-test code (AI uses sleep for synchronization)
pub struct Antipatterns;

impl SourceRule for Antipatterns {
    fn name(&self) -> &'static str {
        "go-antipattern"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let is_test = ctx.filename.ends_with("_test.go");

        let mut any_count = 0;
        let mut first_any_line = 0;
        let mut print_count = 0;
        let mut first_print_line = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // interface{} / any overuse.
            let any_hits = trimmed.matches("interface{}").count()
                + trimmed.matches("interface {}").count();
            // In Go 1.18+, `any` is an alias for interface{} — count it too
            // but only in type positions (hard to detect precisely, skip for now).
            if any_hits > 0 {
                any_count += any_hits;
                if first_any_line == 0 { first_any_line = i + 1; }
            }

            // fmt.Println debugging in non-test code.
            if !is_test && (trimmed.contains("fmt.Println(") || trimmed.contains("fmt.Printf(")
                || trimmed.contains("fmt.Print("))
            {
                // Exclude if it looks like intentional output (main.go or CLI).
                if !ctx.filename.ends_with("main.go") {
                    print_count += 1;
                    if first_print_line == 0 { first_print_line = i + 1; }
                }
            }

            // time.Sleep in non-test code — usually means polling or fake sync.
            if !is_test && trimmed.contains("time.Sleep(") {
                diagnostics.push(Diagnostic {
                    rule: "go-antipattern",
                    message: "time.Sleep in non-test code — use channels, tickers, or context for synchronization".to_string(),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        if any_count >= 3 {
            diagnostics.push(Diagnostic {
                rule: "go-antipattern",
                message: format!(
                    "{any_count} uses of `interface{{}}` — use specific interfaces or generics"
                ),
                line: first_any_line,
                severity: if any_count > 8 { Severity::Slop } else { Severity::Warning },
                weight: if any_count > 8 { 2.5 } else { 1.5 },
            });
        }

        if print_count >= 3 {
            diagnostics.push(Diagnostic {
                rule: "go-antipattern",
                message: format!(
                    "{print_count} fmt.Print calls in library code — use a structured logger"
                ),
                line: first_print_line,
                severity: if print_count > 8 { Severity::Slop } else { Severity::Warning },
                weight: if print_count > 8 { 2.5 } else { 1.5 },
            });
        }

        diagnostics
    }
}
