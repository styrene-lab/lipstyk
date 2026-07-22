use crate::common::comment_analysis;
use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags restating comments in JS/TS — delegates to shared analysis.
pub struct RestatingComments;

impl SourceRule for RestatingComments {
    fn name(&self) -> &'static str {
        "ts-restating-comment"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = comment_analysis::find_restating_comments(
            ctx.source,
            "//",
            "ts-restating-comment",
            |trimmed| {
                let body = trimmed.trim_start_matches('/').trim();
                body.starts_with("!")  // shebangs, ts directives
                    || body.starts_with("TODO")
                    || body.starts_with("FIXME")
                    || body.starts_with("HACK")
                    || body.starts_with("@ts-")
                    || body.starts_with("eslint-")
            },
        );
        diagnostics.extend(find_repeated_declaration_narration(ctx.source));
        diagnostics
    }
}

fn find_repeated_declaration_narration(source: &str) -> Vec<Diagnostic> {
    let lines: Vec<&str> = source.lines().collect();
    let narrated: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let comment = line.trim().strip_prefix("//")?.trim();
            let next_code = lines[index + 1..]
                .iter()
                .map(|candidate| candidate.trim())
                .find(|candidate| !candidate.is_empty() && !candidate.starts_with("//"))?;
            is_declaration_narration(comment, next_code).then_some((index + 1, comment))
        })
        .collect();

    if narrated.len() < 3 {
        return Vec::new();
    }

    vec![Diagnostic {
        rule: "ts-restating-comment",
        message: format!(
            "{} repeated comments narrate declarations like `{}`",
            narrated.len(),
            narrated[0].1
        ),
        line: narrated[0].0,
        severity: Severity::Warning,
        weight: 1.5,
    }]
}

fn is_declaration_narration(comment: &str, code: &str) -> bool {
    let lower = comment.to_ascii_lowercase();
    let prefixes = [
        "allocate ",
        "deallocate ",
        "check if ",
        "get the ",
        "set the ",
        "return the ",
    ];
    let is_declaration = code.contains('(')
        && (code.ends_with('{')
            || code.contains(" =>")
            || code.starts_with("function ")
            || code.starts_with("async function "));
    is_declaration && prefixes.iter().any(|prefix| lower.starts_with(prefix))
}
