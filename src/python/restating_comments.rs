use crate::common::comment_analysis;
use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags restating comments in Python — delegates to shared analysis.
pub struct RestatingComments;

impl SourceRule for RestatingComments {
    fn name(&self) -> &'static str {
        "py-restating-comment"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = comment_analysis::find_restating_comments(
            ctx.source,
            "#",
            "py-restating-comment",
            |trimmed| {
                let body = trimmed.trim_start_matches('#').trim();
                body.starts_with("!")      // shebangs
                    || body.starts_with("type:")  // type: ignore
                    || body.starts_with("noqa")
                    || body.starts_with("TODO")
                    || body.starts_with("FIXME")
                    || body.starts_with("HACK")
                    || body.starts_with("-*-") // encoding declarations
            },
        );
        diagnostics.extend(find_inline_narration(ctx.source));
        diagnostics
    }
}

fn find_inline_narration(source: &str) -> Vec<Diagnostic> {
    let narrated: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let (code, comment) = line.split_once('#')?;
            if code.trim().is_empty() {
                return None;
            }
            let comment = comment.trim();
            is_mechanical_inline_comment(code.trim(), comment).then_some((index + 1, comment))
        })
        .collect();

    if narrated.len() < 3 {
        return Vec::new();
    }

    vec![Diagnostic {
        rule: "py-restating-comment",
        message: format!(
            "{} trailing comments narrate obvious operations like `{}`",
            narrated.len(),
            narrated[0].1
        ),
        line: narrated[0].0,
        severity: Severity::Warning,
        weight: 1.5,
    }]
}

fn is_mechanical_inline_comment(code: &str, comment: &str) -> bool {
    let lower = comment.to_ascii_lowercase();
    if lower.starts_with("type:")
        || lower.starts_with("noqa")
        || lower.starts_with("nosec")
        || comment.is_empty()
    {
        return false;
    }

    (code.contains("input(") && lower.starts_with("reading "))
        || (code.trim_start().starts_with("if ") && lower.contains(" is even"))
        || (code.trim_start().starts_with("else:") && lower.contains(" is odd"))
        || (code.trim_start().starts_with("return ")
            && (lower.starts_with("return ")
                || lower.starts_with("immediately return")
                || lower.starts_with("indicate ")))
}
