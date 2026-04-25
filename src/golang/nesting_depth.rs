use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Go nesting depth — powered by Go AST collector.
pub struct NestingDepth;

const MAX_NESTING: usize = 4;

impl SourceRule for NestingDepth {
    fn name(&self) -> &'static str {
        "go-nesting-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let go = match &ctx.go {
            Some(g) => g,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();

        for f in &go.functions {
            if f.nesting_depth > MAX_NESTING {
                diagnostics.push(Diagnostic {
                    rule: "go-nesting-depth",
                    message: format!(
                        "`{}` has nesting depth {} — extract inner logic into a separate function",
                        f.name, f.nesting_depth
                    ),
                    line: f.line,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}
