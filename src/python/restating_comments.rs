use crate::common::comment_analysis;
use crate::diagnostic::Diagnostic;
use crate::python::{PyContext, PyRule};

/// Flags restating comments in Python — delegates to shared analysis.
pub struct RestatingComments;

impl PyRule for RestatingComments {
    fn name(&self) -> &'static str {
        "py-restating-comment"
    }

    fn check(&self, ctx: &PyContext) -> Vec<Diagnostic> {
        comment_analysis::find_restating_comments(
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
                    || body.starts_with("-*-")    // encoding declarations
            },
        )
    }
}
