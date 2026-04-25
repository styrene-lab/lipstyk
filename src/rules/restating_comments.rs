use crate::common::comment_analysis;
use crate::diagnostic::Diagnostic;
use crate::rules::{LintContext, Rule};

/// Flags comments that just restate what the code does without adding intent.
///
/// Delegates to shared `comment_analysis::find_restating_comments` with
/// Rust-specific skip predicates (doc comments, SAFETY markers, etc.).
pub struct RestatingComments;

impl Rule for RestatingComments {
    fn name(&self) -> &'static str {
        "restating-comment"
    }

    fn check(&self, _file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        comment_analysis::find_restating_comments(
            ctx.source,
            "//",
            "restating-comment",
            |trimmed| {
                trimmed.starts_with("///")
                    || trimmed.starts_with("//!")
                    || trimmed.trim_start_matches('/').trim().starts_with("TODO")
                    || trimmed.trim_start_matches('/').trim().starts_with("FIXME")
                    || trimmed.trim_start_matches('/').trim().starts_with("SAFETY")
                    || trimmed.trim_start_matches('/').trim().starts_with("HACK")
            },
        )
    }
}
