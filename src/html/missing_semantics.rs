use crate::diagnostic::{Diagnostic, Severity};
use crate::html::{HtmlContext, HtmlRule};

/// Flags HTML files that lack semantic elements entirely.
///
/// AI-generated HTML uses `<div>` and `<span>` for everything. A file
/// with many tags but zero semantic elements is a strong signal.
pub struct MissingSemantics;

const SEMANTIC_ELEMENTS: &[&str] = &[
    "main", "nav", "article", "section", "header", "footer",
    "aside", "figure", "figcaption", "details", "summary",
    "dialog", "address", "time", "mark",
];

impl HtmlRule for MissingSemantics {
    fn name(&self) -> &'static str {
        "missing-semantics"
    }

    fn check(&self, ctx: &HtmlContext) -> Vec<Diagnostic> {
        let opening_tags: Vec<_> = ctx.parsed.tags.iter()
            .filter(|t| !t.is_closing)
            .collect();

        if opening_tags.len() < 15 {
            return Vec::new();
        }

        let has_any_semantic = opening_tags.iter()
            .any(|t| SEMANTIC_ELEMENTS.contains(&t.name.as_str()));

        if !has_any_semantic {
            vec![Diagnostic {
                rule: "missing-semantics",
                message: format!(
                    "{} tags but zero semantic elements — \
                     use <main>, <nav>, <section>, <article>, etc.",
                    opening_tags.len()
                ),
                line: 1,
                severity: Severity::Warning,
                weight: 2.0,
            }]
        } else {
            Vec::new()
        }
    }
}
