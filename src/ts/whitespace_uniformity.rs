use crate::common::whitespace;
use crate::diagnostic::Diagnostic;
use crate::source_rule::{Lang, SourceContext, SourceRule};

pub struct WhitespaceUniformity;

impl SourceRule for WhitespaceUniformity {
    fn name(&self) -> &'static str {
        "ts-whitespace-uniformity"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        whitespace::check_whitespace_uniformity(ctx.source, "ts-whitespace-uniformity", 50)
    }
}
