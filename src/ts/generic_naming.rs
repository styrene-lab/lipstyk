use crate::common::naming;
use crate::diagnostic::{Diagnostic, Severity};
use crate::ts::{TsContext, TsRule};

/// Flags generic function names in TS/JS — uses shared name vocabulary.
pub struct GenericNaming;

impl TsRule for GenericNaming {
    fn name(&self) -> &'static str {
        "ts-generic-naming"
    }

    fn check(&self, ctx: &TsContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            for keyword in ["function ", "const ", "let ", "export function ", "async function "] {
                if let Some(rest) = trimmed.strip_prefix(keyword) {
                    let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                        .next()
                        .unwrap_or("");
                    if !name.is_empty() && naming::is_generic_name(name) {
                        diagnostics.push(Diagnostic {
                            rule: "ts-generic-naming",
                            message: format!("`{name}` — name is too vague to convey intent"),
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
