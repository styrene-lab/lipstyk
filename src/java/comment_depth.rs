use crate::common::comment_density;
use crate::diagnostic::Diagnostic;
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Per-function comment density and step narration for Java.
pub struct CommentDepth;

const JAVA_FN_KEYWORDS: &[&str] = &[
    "public ",
    "private ",
    "protected ",
    "static ",
    "public static ",
    "private static ",
    "protected static ",
];

impl SourceRule for CommentDepth {
    fn name(&self) -> &'static str {
        "java-comment-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Java]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(comment_density::check_function_comment_density(
            ctx.source,
            "//",
            "java-comment-depth",
            JAVA_FN_KEYWORDS,
        ));
        diagnostics.extend(comment_density::check_step_narration(
            ctx.source,
            "//",
            "java-comment-depth",
        ));

        diagnostics
    }
}
