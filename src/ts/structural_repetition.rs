use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use crate::treesitter;

/// AST-based structural repetition for TS/JS and Python.
///
/// Same concept as the Rust rule: hash each function's shape (param count,
/// stmt count, control flow pattern) and flag files with high duplication.
/// Uses tree-sitter for real AST parsing instead of text heuristics.
pub struct StructuralRepetition;

impl SourceRule for StructuralRepetition {
    fn name(&self) -> &'static str {
        "ts-structural-repetition"
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
        find_repetitions(&shapes, "ts-structural-repetition")
    }
}

pub struct PyStructuralRepetition;

impl SourceRule for PyStructuralRepetition {
    fn name(&self) -> &'static str {
        "py-structural-repetition"
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
        find_repetitions(&shapes, "py-structural-repetition")
    }
}

fn find_repetitions(shapes: &[treesitter::FnShape], rule_name: &'static str) -> Vec<Diagnostic> {
    if shapes.len() < 4 {
        return Vec::new();
    }

    // Group by shape (excluding name).
    let mut groups: HashMap<(usize, usize, bool, bool, bool), Vec<&treesitter::FnShape>> = HashMap::new();
    for shape in shapes {
        let key = (shape.param_count, shape.stmt_count, shape.has_if, shape.has_for, shape.has_return);
        groups.entry(key).or_default().push(shape);
    }

    let mut diagnostics = Vec::new();
    for fns in groups.values() {
        if fns.len() >= 3 && fns[0].stmt_count > 0 {
            let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
            diagnostics.push(Diagnostic {
                rule: rule_name,
                message: format!(
                    "{} functions share the same shape ({} params, {} stmts): {}",
                    fns.len(), fns[0].param_count, fns[0].stmt_count,
                    names.join(", ")
                ),
                line: fns[0].line,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
    }

    diagnostics
}
