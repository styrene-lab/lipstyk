use crate::common::comment_analysis;
use crate::diagnostic::Diagnostic;
use crate::source_rule::{Lang, SourceContext, SourceRule};

pub struct RestatingComments;

impl SourceRule for RestatingComments {
    fn name(&self) -> &'static str {
        "java-restating-comment"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Java]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
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
