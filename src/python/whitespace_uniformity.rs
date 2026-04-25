use crate::common::whitespace;
use crate::diagnostic::Diagnostic;
use crate::source_rule::{Lang, SourceContext, SourceRule};

pub struct WhitespaceUniformity;

impl SourceRule for WhitespaceUniformity {
    fn name(&self) -> &'static str {
        "py-whitespace-uniformity"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        whitespace::check_whitespace_uniformity(ctx.source, "py-whitespace-uniformity", 50)
    }
}
