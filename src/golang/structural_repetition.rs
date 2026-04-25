use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use crate::treesitter;

/// AST-based structural repetition for Go.
pub struct StructuralRepetition;

impl SourceRule for StructuralRepetition {
    fn name(&self) -> &'static str {
        "go-structural-repetition"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let tree = match treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let shapes = treesitter::extract_fn_shapes(&tree, ctx.source);
        if shapes.len() < 4 {
            return Vec::new();
        }

        let mut groups: HashMap<(usize, usize, bool, bool, bool), Vec<&treesitter::FnShape>> = HashMap::new();
        for shape in &shapes {
            let key = (shape.param_count, shape.stmt_count, shape.has_if, shape.has_for, shape.has_return);
            groups.entry(key).or_default().push(shape);
        }

        let mut diagnostics = Vec::new();
        for fns in groups.values() {
            if fns.len() >= 3 && fns[0].stmt_count > 0 {
                let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
                diagnostics.push(Diagnostic {
                    rule: "go-structural-repetition",
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
