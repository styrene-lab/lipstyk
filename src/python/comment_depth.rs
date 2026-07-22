use crate::common::comment_density;
use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Per-function comment density and step narration for Python.
pub struct CommentDepth;

const PY_FN_KEYWORDS: &[&str] = &["def ", "async def "];

impl SourceRule for CommentDepth {
    fn name(&self) -> &'static str {
        "py-comment-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(comment_density::check_function_comment_density(
            ctx.source,
            "#",
            "py-comment-depth",
            PY_FN_KEYWORDS,
        ));
        diagnostics.extend(comment_density::check_step_narration(
            ctx.source,
            "#",
            "py-comment-depth",
        ));
        diagnostics.extend(check_imperative_narration(ctx.source));

        diagnostics
    }
}

/// Detect a sequence of comments that narrates routine operations rather than
/// recording intent. Requiring three comments in one file keeps isolated
/// headings and explanatory comments quiet.
fn check_imperative_narration(source: &str) -> Vec<Diagnostic> {
    const VERBS: &[&str] = &[
        "add", "calculate", "check", "convert", "create", "determine", "ensure", "extract",
        "find", "get", "increment", "initialize", "iterate", "loop", "print", "process",
        "read", "return", "set", "store", "sum", "try", "update",
    ];

    let narrated: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let body = line.trim().strip_prefix('#')?.trim();
            let first_word = body
                .split(|character: char| !character.is_alphabetic())
                .next()?
                .to_ascii_lowercase();
            VERBS
                .contains(&first_word.as_str())
                .then_some((index + 1, body))
        })
        .collect();

    if narrated.len() < 3 {
        return Vec::new();
    }

    vec![Diagnostic {
        rule: "py-comment-depth",
        message: format!(
            "{} imperative comments narrate routine operations like `{}`",
            narrated.len(),
            narrated[0].1
        ),
        line: narrated[0].0,
        severity: Severity::Warning,
        weight: 1.5,
    }]
}
