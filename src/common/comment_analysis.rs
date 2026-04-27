use crate::diagnostic::{Diagnostic, Severity};

/// Words that signal a comment carries intent or rationale rather than
/// restating what the code does. Shared across all language backends.
pub const INTENT_SIGNALS: &[&str] = &[
    "because",
    "since",
    "so that",
    "ensures",
    "prevents",
    "avoids",
    "workaround",
    "hack",
    "note",
    "nb",
    "important",
    "careful",
    "assumes",
    "invariant",
    "constraint",
    "must",
    "should",
    "otherwise",
    "tradeoff",
    "trade-off",
    "perf",
    "performance",
    "safety",
    "reason",
    "why",
    "intentional",
    "deliberately",
    "temporary",
    "legacy",
    "upstream",
    "see also",
    "cf.",
    "unlike",
    "not",
    "don't",
    "cannot",
    "can't",
    "won't",
    "never",
    "except",
    "unless",
    "despite",
    "although",
    "however",
    "but",
];

/// Check if a comment carries intent signal and should be preserved.
pub fn has_intent_signal(comment: &str) -> bool {
    let lower = comment.to_lowercase();
    INTENT_SIGNALS.iter().any(|s| lower.contains(s))
}

/// Heuristic: check if a comment mostly restates the next line of code.
/// Returns true if >60% of the comment's significant words appear in
/// the code line, and the comment has at least 2 significant words.
pub fn is_restating(comment: &str, code: &str) -> bool {
    let words: Vec<&str> = comment
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() > 2)
        .collect();

    if words.is_empty() {
        return false;
    }

    let code_lower = code.to_lowercase();
    let matches = words
        .iter()
        .filter(|w| code_lower.contains(&w.to_lowercase()))
        .count();

    let ratio = matches as f64 / words.len() as f64;
    ratio >= 0.6 && words.len() >= 2
}

/// Scan source for restating comments. Language-agnostic — caller
/// provides the comment prefix (`//`, `#`, etc.) and skip predicates.
pub fn find_restating_comments(
    source: &str,
    comment_prefix: &str,
    rule_name: &'static str,
    skip_line: impl Fn(&str) -> bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        let comment_text = match trimmed.strip_prefix(comment_prefix) {
            Some(text) => text.trim(),
            None => continue,
        };

        if comment_text.is_empty() || skip_line(trimmed) {
            continue;
        }

        if has_intent_signal(comment_text) {
            continue;
        }

        let next_code = lines[i + 1..]
            .iter()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with(comment_prefix));

        let Some(code_line) = next_code else { continue };

        if is_restating(comment_text, code_line) {
            diagnostics.push(Diagnostic {
                rule: rule_name,
                message: format!("comment restates the code: `{comment_text}`"),
                line: i + 1,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
    }

    diagnostics
}

/// Generic TODO detection. Shared pattern list across all languages.
pub const GENERIC_TODO_PATTERNS: &[&str] = &[
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
