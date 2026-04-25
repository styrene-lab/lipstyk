use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags `async` functions that never `await`.
///
/// AI marks functions async because they might need it, not because
/// they do. An async function without await is just a function that
/// returns a Promise for no reason — it adds overhead and misleads
/// readers about what the function actually does.
///
/// Uses oxc for precise async/await detection.
pub struct RedundantAsync;

impl SourceRule for RedundantAsync {
    fn name(&self) -> &'static str {
        "ts-redundant-async"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let oxc = match ctx.oxc.as_ref() {
            Some(o) => o,
            None => return Vec::new(),
        };

        oxc.functions.iter()
            .filter(|f| f.is_async && !f.has_await)
            .map(|f| {
                let name = if f.name.is_empty() { "<anonymous>" } else { &f.name };
                Diagnostic {
                    rule: "ts-redundant-async",
                    message: format!("`{name}` is async but never awaits"),
                    line: f.line,
                    severity: Severity::Warning,
                    weight: 1.0,
                }
            })
            .collect()
    }
}
