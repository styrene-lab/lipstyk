use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags mutable default arguments: `def f(x=[])`, `def f(x={})`.
///
/// Classic Python gotcha that AI generates routinely. Mutable defaults
/// are shared across calls — the list/dict accumulates state between
/// invocations. The fix is `def f(x=None): x = x or []`.
pub struct MutableDefault;

impl SourceRule for MutableDefault {
    fn name(&self) -> &'static str {
        "py-mutable-default"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            if (trimmed.starts_with("def ") || trimmed.starts_with("async def "))
                && trimmed.contains('(')
            {
                // Check for mutable defaults in the parameter list.
                if let Some(params_start) = trimmed.find('(') {
                    let params_area = &trimmed[params_start..];
                    if params_area.contains("=[]")
                        || params_area.contains("= []")
                        || params_area.contains("={}")
                        || params_area.contains("= {}")
                        || params_area.contains("=set()")
                        || params_area.contains("= set()")
                    {
                        diagnostics.push(Diagnostic {
                            rule: "py-mutable-default",
                            message: "mutable default argument — use `None` and initialize inside the function".to_string(),
                            line: i + 1,
                            severity: Severity::Warning,
                            weight: 1.5,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}
