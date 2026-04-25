use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Go structural repetition — powered by Go AST collector.
pub struct StructuralRepetition;

impl SourceRule for StructuralRepetition {
    fn name(&self) -> &'static str {
        "go-structural-repetition"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let go = match &ctx.go {
            Some(g) => g,
            None => return Vec::new(),
        };

        if go.functions.len() < 4 {
            return Vec::new();
        }

        let mut groups: HashMap<(usize, usize, bool, bool, bool), Vec<&crate::golang::ast::GoFnInfo>> = HashMap::new();
        for f in &go.functions {
            let key = (f.param_count, f.stmt_count, f.has_if, f.has_for, f.has_return);
            groups.entry(key).or_default().push(f);
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
