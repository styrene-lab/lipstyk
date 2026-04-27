use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use crate::treesitter;

/// AST-based trivial wrapper detection for Python.
pub struct TrivialWrapper;

const THRESHOLD: usize = 5;

impl SourceRule for TrivialWrapper {
    fn name(&self) -> &'static str {
        "py-trivial-wrapper"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let tree = match treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let shapes = treesitter::extract_fn_shapes(&tree, ctx.source);
        let wrappers: Vec<&treesitter::FnShape> = shapes
            .iter()
            .filter(|s| s.stmt_count == 1 && !s.name.is_empty())
            .collect();

        if wrappers.len() < THRESHOLD {
            return Vec::new();
        }

        wrappers
            .iter()
            .map(|s| Diagnostic {
                rule: "py-trivial-wrapper",
                message: format!(
                    "`{}` is a single-statement wrapper — does it add value?",
                    s.name
                ),
                line: s.line,
                severity: Severity::Hint,
                weight: 0.75,
            })
            .collect()
    }
}
