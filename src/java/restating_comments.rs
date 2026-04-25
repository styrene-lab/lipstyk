use crate::common::comment_analysis;
use crate::diagnostic::Diagnostic;
use crate::java::{JavaContext, JavaRule};

pub struct RestatingComments;

impl JavaRule for RestatingComments {
    fn name(&self) -> &'static str {
        "java-restating-comment"
    }

    fn check(&self, ctx: &JavaContext) -> Vec<Diagnostic> {
        comment_analysis::find_restating_comments(
            ctx.source,
            "//",
            "java-restating-comment",
            |trimmed| {
                trimmed.starts_with("///")
                    || trimmed.trim_start_matches('/').trim().starts_with("TODO")
                    || trimmed.trim_start_matches('/').trim().starts_with("FIXME")
            },
        )
    }
}
