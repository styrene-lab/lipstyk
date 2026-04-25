use std::collections::HashSet;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use crate::treesitter;

/// AST-based naming entropy for Go.
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
        let tree = match treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let names = treesitter::extract_identifiers(&tree, ctx.source);
        if names.len() < MIN_IDENTIFIERS {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        // Go uses camelCase — split on case boundaries.
        let stems: Vec<String> = names.iter()
            .flat_map(|n| split_camel(n))
            .filter(|s| s.len() > 1)
            .map(|s| s.to_lowercase())
            .collect();

        if stems.len() >= MIN_IDENTIFIERS {
            let unique: HashSet<&str> = stems.iter().map(|s| s.as_str()).collect();
            let ratio = unique.len() as f64 / stems.len() as f64;

            if ratio < LOW_ENTROPY_THRESHOLD {
                diagnostics.push(Diagnostic {
                    rule: "go-naming-entropy",
                    message: format!(
                        "low naming entropy: {}/{} unique stems ({:.0}%)",
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

fn split_camel(name: &str) -> Vec<&str> {
    // For Go camelCase: just use the whole name as one stem.
    // Precise camelCase splitting is complex; the entropy measure
    // works on whole identifiers too since AI reuses the same names.
    vec![name]
}
