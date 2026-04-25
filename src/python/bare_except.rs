use crate::diagnostic::{Diagnostic, Severity};
use crate::python::{PyContext, PyRule};

/// Flags bare `except:` and `except Exception:` that swallow errors.
///
/// AI-generated Python wraps everything in broad try/except to prevent
/// crashes, hiding real bugs. Also catches `pass` in except blocks.
pub struct BareExcept;

impl PyRule for BareExcept {
    fn name(&self) -> &'static str {
        "bare-except"
    }

    fn check(&self, ctx: &PyContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed == "except:" || trimmed == "except Exception:" || trimmed == "except Exception as e:" {
                let next = ctx.source.lines().nth(i + 1).unwrap_or("").trim().to_string();
                let is_swallowed = next == "pass"
                    || next.starts_with("print(")
                    || next.starts_with("logging.")
                    || next.is_empty();

                let (severity, weight) = if trimmed == "except:" {
                    (Severity::Slop, 2.5)
                } else if is_swallowed {
                    (Severity::Warning, 1.5)
                } else {
                    (Severity::Hint, 0.75)
                };

                diagnostics.push(Diagnostic {
                    rule: "bare-except",
                    message: format!(
                        "`{trimmed}` — catch specific exceptions"
                    ),
                    line: i + 1,
                    severity,
                    weight,
                });
            }
        }

        diagnostics
    }
}
