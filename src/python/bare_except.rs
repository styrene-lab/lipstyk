use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags bare `except:` and `except Exception:` that swallow errors.
///
/// AI-generated Python wraps everything in broad try/except to prevent
/// crashes, hiding real bugs. Also catches `pass` in except blocks.
pub struct BareExcept;

impl SourceRule for BareExcept {
    fn name(&self) -> &'static str {
        "bare-except"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed == "except:"
                || trimmed == "except Exception:"
                || trimmed == "except Exception as e:"
            {
                let next = ctx
                    .source
                    .lines()
                    .nth(i + 1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
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
                    message: format!("`{trimmed}` — catch specific exceptions"),
                    line: i + 1,
                    severity,
                    weight,
                });
            }
        }

        diagnostics
    }
}
