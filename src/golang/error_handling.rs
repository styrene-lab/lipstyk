use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Go error handling anti-patterns — now powered by Go AST collector.
pub struct ErrorHandling;

impl SourceRule for ErrorHandling {
    fn name(&self) -> &'static str {
        "go-error-handling"
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

        if go.bare_error_returns.len() >= 3 {
            let count = go.bare_error_returns.len();
            diagnostics.push(Diagnostic {
                rule: "go-error-handling",
                message: format!(
                    "{count} bare `return err` — wrap with fmt.Errorf(\"...: %w\", err) for context"
                ),
                line: go.bare_error_returns[0],
                severity: if count > 8 {
                    Severity::Slop
                } else {
                    Severity::Warning
                },
                weight: if count > 8 { 2.5 } else { 1.5 },
            });
        }

        if go.panic_lines.len() >= 2
            && !ctx.filename.ends_with("_test.go")
            && !ctx.filename.ends_with("main.go")
        {
            diagnostics.push(Diagnostic {
                rule: "go-error-handling",
                message: format!(
                    "{} panic() calls in library code — return errors instead",
                    go.panic_lines.len()
                ),
                line: go.panic_lines[0],
                severity: Severity::Slop,
                weight: 2.5,
            });
        }

        if go.ignored_errors.len() >= 3 {
            diagnostics.push(Diagnostic {
                rule: "go-error-handling",
                message: format!(
                    "{} ignored error returns — handle or propagate errors",
                    go.ignored_errors.len()
                ),
                line: go.ignored_errors[0],
                severity: Severity::Warning,
                weight: 2.0,
            });
        }

        diagnostics
    }
}
