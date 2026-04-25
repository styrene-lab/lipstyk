use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags inline `style=""` attributes on HTML elements.
///
/// AI loves dumping CSS directly into `style` attributes because it
/// doesn't reason about separation of concerns. Uses pre-parsed tags
/// to avoid false-matching `style` inside `<script>` blocks.
pub struct InlineStyles;

impl SourceRule for InlineStyles {
    fn name(&self) -> &'static str {
        "inline-styles"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Html]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let parsed = ctx.html.as_ref().unwrap();
        let hits: Vec<usize> = parsed.tags.iter()
            .filter(|t| !t.is_closing && t.attrs.contains("style="))
            .map(|t| t.line)
            .collect();

        if hits.len() < 3 {
            return Vec::new();
        }

        let (severity, weight) = if hits.len() > 10 {
            (Severity::Slop, 3.0)
        } else {
            (Severity::Warning, 1.5)
        };

        vec![Diagnostic {
            rule: "inline-styles",
            message: format!(
                "{} inline style attributes — extract to CSS classes",
                hits.len()
            ),
            line: hits[0],
            severity,
            weight,
        }]
    }
}
