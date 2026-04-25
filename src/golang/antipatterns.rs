use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Go anti-patterns — powered by Go AST collector.
pub struct Antipatterns;

impl SourceRule for Antipatterns {
    fn name(&self) -> &'static str {
        "go-antipattern"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let go = match &ctx.go {
            Some(g) => g,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();
        let is_test = ctx.filename.ends_with("_test.go");

        // interface{} overuse.
        if go.interface_empty_count >= 3 {
            let count = go.interface_empty_count;
            diagnostics.push(Diagnostic {
                rule: "go-antipattern",
                message: format!(
                    "{count} uses of `interface{{}}` — use specific interfaces or generics"
                ),
                line: 1,
                severity: if count > 8 { Severity::Slop } else { Severity::Warning },
                weight: if count > 8 { 2.5 } else { 1.5 },
            });
        }

        // fmt.Print debugging.
        if !is_test && go.fmt_print_lines.len() >= 3 && !ctx.filename.ends_with("main.go") {
            diagnostics.push(Diagnostic {
                rule: "go-antipattern",
                message: format!(
                    "{} fmt.Print calls in library code — use a structured logger",
                    go.fmt_print_lines.len()
                ),
                line: go.fmt_print_lines[0],
                severity: if go.fmt_print_lines.len() > 8 { Severity::Slop } else { Severity::Warning },
                weight: if go.fmt_print_lines.len() > 8 { 2.5 } else { 1.5 },
            });
        }

        // time.Sleep in non-test code.
        if !is_test {
            for &line in &go.sleep_lines {
                diagnostics.push(Diagnostic {
                    rule: "go-antipattern",
                    message: "time.Sleep in non-test code — use channels, tickers, or context for synchronization".to_string(),
                    line,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        // Functions returning error that never actually return an error.
        let error_fns_without_error: Vec<_> = go.functions.iter()
            .filter(|f| f.returns_error && !f.has_return && f.stmt_count > 0)
            .collect();
        if error_fns_without_error.len() >= 2 {
            for f in &error_fns_without_error {
                diagnostics.push(Diagnostic {
                    rule: "go-antipattern",
                    message: format!(
                        "`{}` returns error but never returns one — remove the error return or use it",
                        f.name
                    ),
                    line: f.line,
                    severity: Severity::Hint,
                    weight: 0.75,
                });
            }
        }

        diagnostics
    }
}
