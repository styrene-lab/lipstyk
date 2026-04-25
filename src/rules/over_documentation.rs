use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};

/// Flags documentation patterns that are AI slop rather than useful context.
///
/// What IS slop:
/// - Step-by-step narration (`// Step 1:`, `// First,`, `// Then,`)
/// - Extreme comment density (>45% of lines are comments)
///
/// What is NOT slop (and we deliberately don't flag):
/// - Doc comments on private functions — in agentic codebases these are
///   navigation aids that save exploration tokens.
/// - Comments explaining *why* — even if verbose, intent is signal.
pub struct OverDocumentation;

impl Rule for OverDocumentation {
    fn name(&self) -> &'static str {
        "over-documentation"
    }

    fn check(&self, _file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_step_comments(ctx.source, &mut diagnostics);
        check_comment_density(ctx.source, &mut diagnostics);
        diagnostics
    }
}

fn check_step_comments(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let step_patterns = [
        "// Step 1",
        "// Step 2",
        "// Step 3",
        "// First,",
        "// Second,",
        "// Third,",
        "// Next,",
        "// Then,",
        "// Finally,",
    ];

    let mut step_count = 0;
    let mut first_line = 0;

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if step_patterns.iter().any(|p| trimmed.starts_with(p)) {
            step_count += 1;
            if step_count == 1 {
                first_line = i + 1;
            }
        }
    }

    if step_count >= 3 {
        diagnostics.push(Diagnostic {
            rule: "over-documentation",
            message: format!(
                "{step_count} step-by-step comments — AI loves narrating code like a tutorial"
            ),
            line: first_line,
            severity: Severity::Slop,
            weight: 3.0,
        });
    }
}

fn check_comment_density(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let total_lines = source.lines().count();
    if total_lines < 30 {
        return;
    }

    let comment_lines = source
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("//") || t.starts_with("/*") || t.starts_with("*")
        })
        .count();

    let ratio = comment_lines as f64 / total_lines as f64;
    if ratio > 0.45 {
        diagnostics.push(Diagnostic {
            rule: "over-documentation",
            message: format!(
                "comment density is {:.0}% — even in well-documented code this is unusually high",
                ratio * 100.0
            ),
            line: 1,
            severity: Severity::Warning,
            weight: 2.0,
        });
    }
}
