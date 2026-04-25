use crate::diagnostic::{Diagnostic, Severity};
use crate::html::{HtmlContext, HtmlRule};

/// Flags excessive `<div>` nesting — the hallmark of AI-generated HTML.
///
/// AI wraps everything in `<div>` because it doesn't reason about
/// semantic elements. We track actual nesting depth through the tag
/// tree rather than resetting on non-div tags.
pub struct DivSoup;

impl HtmlRule for DivSoup {
    fn name(&self) -> &'static str {
        "div-soup"
    }

    fn check(&self, ctx: &HtmlContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let opening_tags: Vec<_> = ctx.parsed.tags.iter()
            .filter(|t| !t.is_closing && !t.is_self_closing)
            .collect();

        let total = opening_tags.len();
        let div_count = opening_tags.iter().filter(|t| t.name == "div").count();

        if total >= 10 {
            let ratio = div_count as f64 / total as f64;
            if ratio > 0.5 {
                diagnostics.push(Diagnostic {
                    rule: "div-soup",
                    message: format!(
                        "{div_count}/{total} opening tags are <div> ({:.0}%) — use semantic elements",
                        ratio * 100.0
                    ),
                    line: 1,
                    severity: Severity::Warning,
                    weight: 2.5,
                });
            }
        }

        // Track actual div nesting depth through the tag tree.
        let mut div_depth = 0usize;
        let mut max_depth = 0usize;
        let mut max_depth_line = 1;

        for tag in &ctx.parsed.tags {
            if tag.name == "div" {
                if tag.is_closing {
                    div_depth = div_depth.saturating_sub(1);
                } else if !tag.is_self_closing {
                    div_depth += 1;
                    if div_depth > max_depth {
                        max_depth = div_depth;
                        max_depth_line = tag.line;
                    }
                }
            }
        }

        if max_depth >= 5 {
            diagnostics.push(Diagnostic {
                rule: "div-soup",
                message: format!(
                    "{max_depth} levels of nested <div> — consider <section>, <article>, <nav>, etc."
                ),
                line: max_depth_line,
                severity: Severity::Slop,
                weight: 3.0,
            });
        }

        diagnostics
    }
}
