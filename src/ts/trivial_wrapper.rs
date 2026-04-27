use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// AST-based trivial wrapper detection for TS/JS.
///
/// Flags functions whose body is a single statement.
/// Uses oxc for function shape extraction.
pub struct TrivialWrapper;

const THRESHOLD: usize = 5;

impl SourceRule for TrivialWrapper {
    fn name(&self) -> &'static str {
        "ts-trivial-wrapper"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let oxc = match ctx.oxc.as_ref() {
            Some(o) => o,
            None => return Vec::new(),
        };

        let wrappers: Vec<_> = oxc
            .functions
            .iter()
            .filter(|f| f.stmt_count == 1 && !f.name.is_empty())
            .collect();

        if wrappers.len() < THRESHOLD {
            return Vec::new();
        }

        wrappers
            .iter()
            .map(|f| Diagnostic {
                rule: "ts-trivial-wrapper",
                message: format!(
                    "`{}` is a single-statement wrapper — does it add value?",
                    f.name
                ),
                line: f.line,
                severity: Severity::Hint,
                weight: 0.75,
            })
            .collect()
    }
}
