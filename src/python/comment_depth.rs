use crate::common::comment_density;
use crate::diagnostic::Diagnostic;
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
            ctx.source, "#", "py-comment-depth", PY_FN_KEYWORDS,
        ));
        diagnostics.extend(comment_density::check_step_narration(
            ctx.source, "#", "py-comment-depth",
        ));

        diagnostics
    }
}
