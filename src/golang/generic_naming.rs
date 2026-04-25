use crate::common::naming;
use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

pub struct GenericNaming;

impl SourceRule for GenericNaming {
    fn name(&self) -> &'static str {
        "go-generic-naming"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            // Match func declarations.
            if let Some(rest) = trimmed.strip_prefix("func ") {
                // Skip method receivers: func (r *Receiver) Name(...)
                let name_part = if rest.starts_with('(') {
                    // Method: skip past the receiver.
                    rest.split(')').nth(1).unwrap_or("").trim()
                } else {
                    rest
                };

                let name = name_part.split('(').next().unwrap_or("").trim();
                if !name.is_empty() && naming::is_generic_name(name) {
                    diagnostics.push(Diagnostic {
                        rule: "go-generic-naming",
                        message: format!("`func {name}` — name is too vague to convey intent"),
                        line: i + 1,
                        severity: Severity::Warning,
                        weight: 1.5,
                    });
                }
            }
        }

        diagnostics
    }
}
