use crate::common::naming;
use crate::diagnostic::{Diagnostic, Severity};
use crate::python::{PyContext, PyRule};

/// Flags generic function names in Python — uses shared name vocabulary.
pub struct GenericNaming;

impl PyRule for GenericNaming {
    fn name(&self) -> &'static str {
        "py-generic-naming"
    }

    fn check(&self, ctx: &PyContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            let def_line = trimmed.strip_prefix("def ")
                .or_else(|| trimmed.strip_prefix("async def "));

            if let Some(rest) = def_line {
                let name = rest.split('(').next().unwrap_or("").trim();
                if !name.is_empty() && naming::is_generic_name(name) {
                    diagnostics.push(Diagnostic {
                        rule: "py-generic-naming",
                        message: format!("`def {name}` — name is too vague to convey intent"),
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
