use crate::common::whitespace;
use crate::diagnostic::Diagnostic;
use crate::rules::{LintContext, Rule};

/// Rust-specific whitespace uniformity — delegates to shared analysis.
pub struct WhitespaceUniformity;

impl Rule for WhitespaceUniformity {
    fn name(&self) -> &'static str {
        "whitespace-uniformity"
    }

    fn check(&self, _file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        whitespace::check_whitespace_uniformity(ctx.source, "whitespace-uniformity", 50)
    }
}
