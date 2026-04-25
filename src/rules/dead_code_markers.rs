use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};

/// Flags high density of `#[allow(dead_code)]` and `#[allow(unused)]` annotations.
///
/// AI generates code from abandoned approaches and doesn't clean up.
/// A file with many dead-code suppressions signals code that was generated
/// without intent — the AI tried something, it didn't work, and it
/// papered over the warnings instead of removing the code.
pub struct DeadCodeMarkers;

const ALLOW_PATTERNS: &[&str] = &[
    "#[allow(dead_code)]",
    "#[allow(unused)]",
    "#[allow(unused_variables)]",
    "#[allow(unused_imports)]",
    "#[allow(unused_mut)]",
    "#[allow(unused_assignments)]",
];

impl Rule for DeadCodeMarkers {
    fn name(&self) -> &'static str {
        "dead-code-markers"
    }

    fn check(&self, _file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut count = 0;
        let mut first_line = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[") && ALLOW_PATTERNS.iter().any(|p| trimmed.contains(p)) {
                count += 1;
                if first_line == 0 {
                    first_line = i + 1;
                }
            }
        }

        if count >= 3 {
            vec![Diagnostic {
                rule: "dead-code-markers",
                message: format!(
                    "{count} dead-code suppressions — remove unused code instead of silencing warnings"
                ),
                line: first_line,
                severity: Severity::Warning,
                weight: 1.5,
            }]
        } else {
            Vec::new()
        }
    }
}
