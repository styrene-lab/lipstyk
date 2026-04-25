use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// AST-based structural repetition for TS/JS — powered by oxc.
pub struct StructuralRepetition;

impl SourceRule for StructuralRepetition {
    fn name(&self) -> &'static str {
        "ts-structural-repetition"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let parsed = match &ctx.oxc {
            Some(p) => p,
            None => return Vec::new(),
        };

        if parsed.functions.len() < 4 {
            return Vec::new();
        }

        let mut groups: HashMap<(usize, usize, bool, bool, bool), Vec<&crate::oxc::FnInfo>> = HashMap::new();
        for f in &parsed.functions {
            let key = (f.param_count, f.stmt_count, f.has_if, f.has_for, f.has_return);
            groups.entry(key).or_default().push(f);
        }

        let mut diagnostics = Vec::new();
        for fns in groups.values() {
            if fns.len() >= 3 && fns[0].stmt_count > 0 {
                let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
                diagnostics.push(Diagnostic {
                    rule: "ts-structural-repetition",
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
}

/// Python still uses tree-sitter.
pub struct PyStructuralRepetition;

impl SourceRule for PyStructuralRepetition {
    fn name(&self) -> &'static str {
        "py-structural-repetition"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let tree = match crate::treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let shapes = crate::treesitter::extract_fn_shapes(&tree, ctx.source);
        if shapes.len() < 4 {
            return Vec::new();
        }

        let mut groups: HashMap<(usize, usize, bool, bool, bool), Vec<&crate::treesitter::FnShape>> = HashMap::new();
        for shape in &shapes {
            let key = (shape.param_count, shape.stmt_count, shape.has_if, shape.has_for, shape.has_return);
            groups.entry(key).or_default().push(shape);
        }

        let mut diagnostics = Vec::new();
        for fns in groups.values() {
            if fns.len() >= 3 && fns[0].stmt_count > 0 {
                let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
                diagnostics.push(Diagnostic {
                    rule: "py-structural-repetition",
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
}
