use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use crate::treesitter;

/// AST-based trivial wrapper detection for TS/JS.
///
/// Same concept as the Rust rule: flags functions whose body is a single
/// return/expression statement. Uses tree-sitter to count actual AST
/// statements rather than guessing from text.
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
        let tree = match treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let shapes = treesitter::extract_fn_shapes(&tree, ctx.source);
        let wrappers: Vec<&treesitter::FnShape> = shapes.iter()
            .filter(|s| s.stmt_count == 1 && !s.name.is_empty())
            .collect();

        if wrappers.len() < THRESHOLD {
            return Vec::new();
        }

        wrappers.iter().map(|s| {
            Diagnostic {
                rule: "ts-trivial-wrapper",
                message: format!(
                    "`{}` is a single-statement wrapper — does it add value?",
                    s.name
                ),
                line: s.line,
                severity: Severity::Hint,
                weight: 0.75,
            }
        }).collect()
    }
}
