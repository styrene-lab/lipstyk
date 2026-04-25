use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags placeholder and boilerplate content in Markdown.
///
/// AI scaffolding leaves "your-project", "TODO: add description",
/// and generic opening paragraphs that could describe anything.
pub struct Placeholders;

const PLACEHOLDER_PATTERNS: &[&str] = &[
    "your-project", "your-app", "your-name", "your-username",
    "your-api-key", "your-token", "your project", "your app",
    "insert ", "replace with", "add description here",
    "description here", "enter your", "fill in",
];

const GENERIC_OPENERS: &[&str] = &[
    "a comprehensive",
    "this project is a",
    "this is a simple",
    "this repository contains",
    "welcome to the",
    "this tool provides",
    "this library offers",
    "this package is designed",
];

impl SourceRule for Placeholders {
    fn name(&self) -> &'static str {
        "md-placeholder"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Markdown]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut placeholder_count = 0;
        let mut first_placeholder = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let lower = line.to_lowercase();

            for pattern in PLACEHOLDER_PATTERNS {
                if lower.contains(pattern) {
                    placeholder_count += 1;
                    if first_placeholder == 0 {
                        first_placeholder = i + 1;
                    }
                    break;
                }
            }
        }

        if placeholder_count >= 2 {
            diagnostics.push(Diagnostic {
                rule: "md-placeholder",
                message: format!(
                    "{placeholder_count} placeholder strings — fill in or remove template content"
                ),
                line: first_placeholder,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }

        // Check for generic opening paragraph (first 5 non-heading, non-empty lines).
        let opening: String = ctx.source.lines()
            .filter(|l| !l.trim().starts_with('#') && !l.trim().is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        for opener in GENERIC_OPENERS {
            if opening.contains(opener) {
                diagnostics.push(Diagnostic {
                    rule: "md-placeholder",
                    message: format!(
                        "generic opening paragraph (\"{opener}...\") — write something specific to this project"
                    ),
                    line: 1,
                    severity: Severity::Hint,
                    weight: 1.0,
                });
                break;
            }
        }

        diagnostics
    }
}
