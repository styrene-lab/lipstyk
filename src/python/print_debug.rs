use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags `print()` debugging left in production code.
///
/// Same signal as console.log in JS — AI uses print() for debugging
/// and doesn't clean up. Exempts files that look like CLI scripts
/// (have `if __name__` or `argparse`).
pub struct PrintDebug;

impl SourceRule for PrintDebug {
    fn name(&self) -> &'static str {
        "print-debug"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        // Exempt CLI scripts.
        if ctx.source.contains("if __name__") || ctx.source.contains("argparse") {
            return Vec::new();
        }

        let mut hits = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("print(") || trimmed.contains(" print(") {
                hits.push(i + 1);
            }
        }

        if hits.len() < 3 {
            return Vec::new();
        }

        vec![Diagnostic {
            rule: "print-debug",
            message: format!(
                "{} print() calls — use logging module or remove debug output",
                hits.len()
            ),
            line: hits[0],
            severity: if hits.len() > 10 { Severity::Slop } else { Severity::Warning },
            weight: if hits.len() > 10 { 3.0 } else { 1.5 },
        }]
    }
}
