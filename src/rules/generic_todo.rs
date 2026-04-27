use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};

/// Flags generic AI-style TODO comments.
///
/// AI TODOs are vague and formulaic: "TODO: Add error handling",
/// "TODO: Implement this", "TODO: Add logging". Human TODOs tend
/// to be specific: "TODO(wilson): handle negative offsets in range calc".
pub struct GenericTodo;

const GENERIC_TODO_PATTERNS: &[&str] = &[
    "add error handling",
    "add proper error handling",
    "handle error",
    "handle errors",
    "implement this",
    "implement later",
    "add logging",
    "add proper logging",
    "add validation",
    "validate input",
    "add input validation",
    "add tests",
    "write tests",
    "add unit tests",
    "clean up",
    "clean this up",
    "refactor this",
    "refactor later",
    "optimize this",
    "optimize later",
    "improve this",
    "fix this",
    "fix later",
    "make this better",
    "add documentation",
    "add proper documentation",
    "handle edge cases",
    "handle edge case",
    "add type checking",
    "implement properly",
    "add retry logic",
    "add caching",
];

impl Rule for GenericTodo {
    fn name(&self) -> &'static str {
        "generic-todo"
    }

    fn check(&self, _file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            // Match // TODO, // FIXME, // HACK with generic content.
            let comment_text = if let Some(rest) = trimmed.strip_prefix("//") {
                rest.trim()
            } else {
                continue;
            };

            let todo_body = if let Some(rest) = comment_text.strip_prefix("TODO") {
                rest.trim_start_matches([':', ' '])
            } else if let Some(rest) = comment_text.strip_prefix("FIXME") {
                rest.trim_start_matches([':', ' '])
            } else {
                continue;
            };

            let lower = todo_body.to_lowercase();
            if GENERIC_TODO_PATTERNS.iter().any(|p| lower.starts_with(p)) {
                diagnostics.push(Diagnostic {
                    rule: "generic-todo",
                    message: format!("generic TODO: `{todo_body}` — AI TODOs lack specificity"),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}
