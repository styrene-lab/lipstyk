use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags bare `catch (Exception e)` blocks — Java's equivalent of
/// Python's bare `except:`. AI catches Exception and does nothing
/// useful with it.
pub struct BareCatch;

impl SourceRule for BareCatch {
    fn name(&self) -> &'static str {
        "java-bare-catch"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Java]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("catch") && trimmed.contains("Exception") {
                let next = ctx
                    .source
                    .lines()
                    .nth(i + 1)
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let is_swallowed = next.is_empty()
                    || next == "}"
                    || next.starts_with("//")
                    || next.starts_with("e.printStackTrace()")
                    || next.starts_with("System.out.print")
                    || next.starts_with("log.");

                if is_swallowed {
                    diagnostics.push(Diagnostic {
                        rule: "java-bare-catch",
                        message: format!("`{trimmed}` — catch specific exceptions and handle them"),
                        line: i + 1,
                        severity: Severity::Warning,
                        weight: 1.5,
                    });
                }
            }
        }

        diagnostics
    }
}
