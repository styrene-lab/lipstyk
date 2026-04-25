use crate::common::comment_density;
use crate::diagnostic::Diagnostic;
use crate::source_rule::{Lang, SourceContext, SourceRule};

pub struct CommentDepth;

const GO_FN_KEYWORDS: &[&str] = &["func "];

impl SourceRule for CommentDepth {
    fn name(&self) -> &'static str {
        "go-comment-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.extend(comment_density::check_function_comment_density(
            ctx.source, "//", "go-comment-depth", GO_FN_KEYWORDS,
        ));
        diagnostics.extend(comment_density::check_step_narration(
            ctx.source, "//", "go-comment-depth",
        ));
        diagnostics
    }
}
