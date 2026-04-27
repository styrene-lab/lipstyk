use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags Promise anti-patterns common in AI-generated JS/TS.
///
/// - `new Promise` wrapping an already-async operation (the explicit
///   constructor anti-pattern)
/// - `.then().catch()` chains when `async`/`await` would be clearer
/// - Bare `.catch(() => {})` that swallows errors silently
pub struct PromiseAntipattern;

impl SourceRule for PromiseAntipattern {
    fn name(&self) -> &'static str {
        "promise-antipattern"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut then_catch_count = 0;
        let mut first_then_catch = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            // Explicit constructor anti-pattern. Event-emitter wrappers legitimately need
            // a Promise constructor because there is no awaitable API to call directly.
            if trimmed.contains("new Promise(") && !ctx.source.contains(".on(") {
                diagnostics.push(Diagnostic {
                    rule: "promise-antipattern",
                    message: "`new Promise()` — can this use async/await instead?".to_string(),
                    line: i + 1,
                    severity: Severity::Hint,
                    weight: 0.75,
                });
            }

            // .then().catch() chain.
            if trimmed.contains(".then(") {
                then_catch_count += 1;
                if first_then_catch == 0 {
                    first_then_catch = i + 1;
                }
            }

            // Silent error swallowing.
            if trimmed.contains(".catch(() => {})")
                || trimmed.contains(".catch(() => null)")
                || trimmed.contains(".catch(()=>{})")
                || trimmed.contains(".catch(e => {})")
            {
                diagnostics.push(Diagnostic {
                    rule: "promise-antipattern",
                    message: "`.catch(() => {})` silently swallows errors".to_string(),
                    line: i + 1,
                    severity: Severity::Slop,
                    weight: 2.5,
                });
            }
        }

        if then_catch_count >= 3 {
            diagnostics.push(Diagnostic {
                rule: "promise-antipattern",
                message: format!(
                    "{then_catch_count} `.then()` chains — prefer async/await for readability"
                ),
                line: first_then_catch,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }

        diagnostics
    }
}
