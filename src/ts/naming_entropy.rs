use std::collections::HashSet;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use crate::treesitter;

/// AST-based naming entropy for TS/JS and Python.
///
/// Same concept as the Rust rule: measure the ratio of unique name stems
/// to total identifiers. Low ratio = repetitive AI naming vocabulary.
/// Uses oxc for TS/JS, tree-sitter for Python.
pub struct NamingEntropy;

const MIN_IDENTIFIERS: usize = 15;
const LOW_ENTROPY_THRESHOLD: f64 = 0.35;
const VERBOSE_FLOOR: usize = 12;

impl SourceRule for NamingEntropy {
    fn name(&self) -> &'static str {
        "ts-naming-entropy"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let Some(oxc) = ctx.oxc.as_ref() else {
            return Vec::new();
        };
        analyze_entropy(&oxc.identifiers, "ts-naming-entropy")
    }
}

pub struct PyNamingEntropy;

impl SourceRule for PyNamingEntropy {
    fn name(&self) -> &'static str {
        "py-naming-entropy"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let tree = match treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let names = treesitter::extract_identifiers(&tree, ctx.source);
        analyze_entropy(&names, "py-naming-entropy")
    }
}

fn analyze_entropy(names: &[String], rule_name: &'static str) -> Vec<Diagnostic> {
    if names.len() < MIN_IDENTIFIERS {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();

    // Split camelCase and snake_case into stems.
    let stems: Vec<&str> = names
        .iter()
        .flat_map(|n| split_name(n))
        .filter(|s| s.len() > 1)
        .collect();

    if stems.len() >= MIN_IDENTIFIERS {
        let unique: HashSet<&&str> = stems.iter().collect();
        let ratio = unique.len() as f64 / stems.len() as f64;

        if ratio < LOW_ENTROPY_THRESHOLD {
            diagnostics.push(Diagnostic {
                rule: rule_name,
                message: format!(
                    "low naming entropy: {}/{} unique stems ({:.0}%)",
                    unique.len(),
                    stems.len(),
                    ratio * 100.0
                ),
                line: 1,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
    }

    // Uniform verbosity check.
    let mut lengths: Vec<usize> = names.iter().map(|n| n.len()).collect();
    if lengths.len() >= MIN_IDENTIFIERS {
        let short_count = lengths.iter().filter(|&&l| l <= 4).count();
        lengths.sort();
        let median = lengths[lengths.len() / 2];

        if median >= VERBOSE_FLOOR && short_count == 0 {
            diagnostics.push(Diagnostic {
                rule: rule_name,
                message: format!(
                    "uniformly verbose naming: median length {median}, zero short names"
                ),
                line: 1,
                severity: Severity::Hint,
                weight: 0.75,
            });
        }
    }

    diagnostics
}

fn split_name(name: &str) -> Vec<&str> {
    // Split on underscores (snake_case).
    if name.contains('_') {
        return name.split('_').filter(|s| !s.is_empty()).collect();
    }

    // Split on camelCase boundaries — just collect the whole name as one stem
    // since precise camelCase splitting requires more logic than it's worth here.
    vec![name]
}
