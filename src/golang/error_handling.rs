use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags Go error handling anti-patterns.
///
/// Go's explicit error handling is one of its defining features, and AI
/// routinely gets it wrong:
/// - Bare `return err` without wrapping (`fmt.Errorf("...: %w", err)`)
/// - `panic()` in library code (should return error)
/// - `_ = someFunc()` ignoring returned errors
/// - Empty error checks: `if err != nil { }` or `if err != nil { return nil }`
pub struct ErrorHandling;

impl SourceRule for ErrorHandling {
    fn name(&self) -> &'static str {
        "go-error-handling"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        let mut bare_return_err = 0;
        let mut first_bare_line = 0;
        let mut panic_count = 0;
        let mut first_panic_line = 0;
        let mut ignored_errors = 0;
        let mut first_ignored_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Bare return err (no wrapping).
            if trimmed == "return err" || trimmed == "return nil, err"
                || trimmed == "return \"\", err" || trimmed == "return 0, err"
                || trimmed == "return false, err"
            {
                bare_return_err += 1;
                if first_bare_line == 0 { first_bare_line = i + 1; }
            }

            // panic() in non-test, non-main code.
            if (trimmed.starts_with("panic(") || trimmed.contains(" panic("))
                && !ctx.filename.ends_with("_test.go")
                && !ctx.filename.ends_with("main.go")
            {
                panic_count += 1;
                if first_panic_line == 0 { first_panic_line = i + 1; }
            }

            // Ignored error: _ = someFunc() or _ , _ = ...
            if trimmed.starts_with("_ =") || trimmed.starts_with("_, _ =")
                || trimmed.contains("_ = ")
            {
                // Check if the RHS likely returns an error.
                if trimmed.contains("(") {
                    ignored_errors += 1;
                    if first_ignored_line == 0 { first_ignored_line = i + 1; }
                }
            }
        }

        if bare_return_err >= 3 {
            diagnostics.push(Diagnostic {
                rule: "go-error-handling",
                message: format!(
                    "{bare_return_err} bare `return err` — wrap with fmt.Errorf(\"...: %w\", err) for context"
                ),
                line: first_bare_line,
                severity: if bare_return_err > 8 { Severity::Slop } else { Severity::Warning },
                weight: if bare_return_err > 8 { 2.5 } else { 1.5 },
            });
        }

        if panic_count >= 2 {
            diagnostics.push(Diagnostic {
                rule: "go-error-handling",
                message: format!(
                    "{panic_count} panic() calls in library code — return errors instead"
                ),
                line: first_panic_line,
                severity: Severity::Slop,
                weight: 2.5,
            });
        }

        if ignored_errors >= 3 {
            diagnostics.push(Diagnostic {
                rule: "go-error-handling",
                message: format!(
                    "{ignored_errors} ignored error returns (_ = ...) — handle or propagate errors"
                ),
                line: first_ignored_line,
                severity: Severity::Warning,
                weight: 2.0,
            });
        }

        diagnostics
    }
}
