use crate::common::naming;
use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

pub struct GenericNaming;

impl SourceRule for GenericNaming {
    fn name(&self) -> &'static str {
        "java-generic-naming"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Java]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            // Match method declarations.
            for vis in ["public ", "private ", "protected ", "static ", ""] {
                if let Some(rest) = trimmed.strip_prefix(vis) {
                    if rest.contains('(')
                        && !rest.starts_with("class ")
                        && !rest.starts_with("interface ")
                        && !rest.starts_with("if ")
                        && !rest.starts_with("for ")
                        && !rest.starts_with("while ")
                        && !rest.starts_with("new ")
                        && !rest.starts_with("return ")
                    {
                        // Extract method name: last word before (
                        let before_paren = rest.split('(').next().unwrap_or("");
                        let name = before_paren.split_whitespace().last().unwrap_or("");
                        if !name.is_empty() && naming::is_generic_name(name) {
                            diagnostics.push(Diagnostic {
                                rule: "java-generic-naming",
                                message: format!("`{name}` — name is too vague to convey intent"),
                                line: i + 1,
                                severity: Severity::Warning,
                                weight: 1.5,
                            });
                        }
                    }
                    break;
                }
            }
        }

        diagnostics
    }
}
