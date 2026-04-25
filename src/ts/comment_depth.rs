use crate::common::comment_density;
use crate::diagnostic::Diagnostic;
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Per-function comment density and step narration for TS/JS.
pub struct CommentDepth;

const TS_FN_KEYWORDS: &[&str] = &[
    "function ", "async function ", "export function ",
    "export async function ", "export default function ",
];

impl SourceRule for CommentDepth {
    fn name(&self) -> &'static str {
        "ts-comment-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(comment_density::check_function_comment_density(
            ctx.source, "//", "ts-comment-depth", TS_FN_KEYWORDS,
        ));
        diagnostics.extend(comment_density::check_step_narration(
            ctx.source, "//", "ts-comment-depth",
        ));

        diagnostics
    }
}
