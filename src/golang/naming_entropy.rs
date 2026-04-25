use std::collections::HashSet;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Go naming entropy — powered by Go AST collector.
pub struct NamingEntropy;

const MIN_IDENTIFIERS: usize = 15;
const LOW_ENTROPY_THRESHOLD: f64 = 0.35;

impl SourceRule for NamingEntropy {
    fn name(&self) -> &'static str {
        "go-naming-entropy"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let go = match &ctx.go {
            Some(g) => g,
            None => return Vec::new(),
        };

        let names = &go.identifiers;
        if names.len() < MIN_IDENTIFIERS {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        // Go uses camelCase — use whole names as stems.
        let stems: Vec<&str> = names.iter()
            .map(|n| n.as_str())
            .filter(|s| s.len() > 1)
            .collect();

        if stems.len() >= MIN_IDENTIFIERS {
            let unique: HashSet<&str> = stems.iter().copied().collect();
            let ratio = unique.len() as f64 / stems.len() as f64;

            if ratio < LOW_ENTROPY_THRESHOLD {
                diagnostics.push(Diagnostic {
                    rule: "go-naming-entropy",
                    message: format!(
                        "low naming entropy: {}/{} unique identifiers ({:.0}%)",
                        unique.len(), stems.len(), ratio * 100.0
                    ),
                    line: 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}
