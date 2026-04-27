use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags inconsistent type hint usage.
///
/// AI either type-hints everything (including obvious literals) or
/// nothing. The tell is inconsistency within a file — some functions
/// fully annotated, others completely bare. Also flags `-> None` on
/// functions that clearly return None (every function without a return
/// "returns None" — annotating it is noise).
pub struct TypeHintGaps;

impl SourceRule for TypeHintGaps {
    fn name(&self) -> &'static str {
        "type-hint-gaps"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut hinted = 0;
        let mut unhinted = 0;

        for line in ctx.source.lines() {
            let trimmed = line.trim();

            // Match `def foo(` lines.
            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                if trimmed.contains("->") {
                    hinted += 1;
                } else {
                    unhinted += 1;
                }
            }
        }

        let total = hinted + unhinted;
        if total < 4 {
            return Vec::new();
        }

        // Flag inconsistency: some hinted, some not.
        let hinted_ratio = hinted as f64 / total as f64;
        if hinted_ratio > 0.2 && hinted_ratio < 0.8 {
            return vec![Diagnostic {
                rule: "type-hint-gaps",
                message: format!("{hinted}/{total} functions have type hints — be consistent"),
                line: 1,
                severity: Severity::Hint,
                weight: 1.0,
            }];
        }

        Vec::new()
    }
}
