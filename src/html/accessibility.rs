use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags accessibility gaps that AI-generated HTML commonly produces.
///
/// Uses pre-parsed tags so multi-line attributes are handled correctly.
pub struct Accessibility;

impl SourceRule for Accessibility {
    fn name(&self) -> &'static str {
        "accessibility"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Html]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let parsed = ctx.html.as_ref().unwrap();
        let mut diagnostics = Vec::new();

        // <img> without alt
        let missing_alt: Vec<usize> = parsed.tags.iter()
            .filter(|t| t.name == "img" && !t.attrs.to_lowercase().contains("alt="))
            .map(|t| t.line)
            .collect();

        if !missing_alt.is_empty() {
            diagnostics.push(Diagnostic {
                rule: "accessibility",
                message: format!("{} <img> tag(s) missing `alt` attribute", missing_alt.len()),
                line: missing_alt[0],
                severity: if missing_alt.len() > 3 { Severity::Slop } else { Severity::Warning },
                weight: if missing_alt.len() > 3 { 3.0 } else { 1.5 },
            });
        }

        // <html> without lang
        let html_tag = parsed.tags.iter().find(|t| t.name == "html" && !t.is_closing);
        if let Some(tag) = html_tag
            && !tag.attrs.to_lowercase().contains("lang=") {
                diagnostics.push(Diagnostic {
                    rule: "accessibility",
                    message: "<html> missing `lang` attribute".to_string(),
                    line: tag.line,
                    severity: Severity::Warning,
                    weight: 1.0,
                });
            }

        // <button> with no text content and no aria-label (best-effort single-line check)
        let empty_buttons: Vec<usize> = parsed.tags.iter()
            .filter(|t| {
                t.name == "button" && !t.is_closing
                    && !t.attrs.to_lowercase().contains("aria-label")
            })
            .filter(|t| {
                // Check if the line has visible text after the tag close.
                let line = ctx.source.lines().nth(t.line.saturating_sub(1)).unwrap_or("");
                if let Some(gt) = line.find('>') {
                    let after = &line[gt + 1..];
                    let text = if let Some(close) = after.find("</") {
                        after[..close].trim()
                    } else {
                        after.trim()
                    };
                    text.is_empty() || text.starts_with('<')
                } else {
                    false
                }
            })
            .map(|t| t.line)
            .collect();

        if !empty_buttons.is_empty() {
            diagnostics.push(Diagnostic {
                rule: "accessibility",
                message: format!(
                    "{} <button> element(s) with no visible text or aria-label",
                    empty_buttons.len()
                ),
                line: empty_buttons[0],
                severity: Severity::Warning,
                weight: 1.5,
            });
        }

        diagnostics
    }
}
