use crate::diagnostic::{Diagnostic, Severity};
use crate::python::{PyContext, PyRule};

/// Flags `from X import *` — AI doesn't reason about namespaces.
///
/// Wildcard imports pollute the namespace and make it impossible to
/// tell where names come from. Also catches excessive `import` blocks
/// (20+ imports suggests the AI pulled in everything it might need).
pub struct ImportStar;

impl PyRule for ImportStar {
    fn name(&self) -> &'static str {
        "import-star"
    }

    fn check(&self, ctx: &PyContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut import_count = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("from ") && trimmed.contains("import *") {
                diagnostics.push(Diagnostic {
                    rule: "import-star",
                    message: format!("`{trimmed}` — import specific names"),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }

            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                import_count += 1;
            }
        }

        if import_count >= 20 {
            diagnostics.push(Diagnostic {
                rule: "import-star",
                message: format!(
                    "{import_count} imports — are all of these used?"
                ),
                line: 1,
                severity: Severity::Hint,
                weight: 0.75,
            });
        }

        diagnostics
    }
}
