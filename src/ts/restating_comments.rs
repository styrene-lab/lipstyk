use crate::common::comment_analysis;
use crate::diagnostic::Diagnostic;
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
        comment_analysis::find_restating_comments(
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
        )
    }
}
