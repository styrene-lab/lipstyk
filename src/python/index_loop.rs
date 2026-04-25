use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags `for i in range(len(x))` — Python's equivalent of the
/// C-style index loop.
///
/// AI defaults to index-based iteration instead of `for item in x`
/// or `for i, item in enumerate(x)`. Text-based detection is reliable
/// here since the pattern is syntactically distinctive.
pub struct IndexLoop;

impl SourceRule for IndexLoop {
    fn name(&self) -> &'static str {
        "py-index-loop"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ") && trimmed.contains("range(len(") {
                diagnostics.push(Diagnostic {
                    rule: "py-index-loop",
                    message: "C-style `for i in range(len(x))` — use `for item in x` or `enumerate()`".to_string(),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}
