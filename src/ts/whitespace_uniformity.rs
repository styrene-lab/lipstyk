use crate::common::whitespace;
use crate::diagnostic::Diagnostic;
use crate::ts::{TsContext, TsRule};

pub struct WhitespaceUniformity;

impl TsRule for WhitespaceUniformity {
    fn name(&self) -> &'static str {
        "ts-whitespace-uniformity"
    }

    fn check(&self, ctx: &TsContext) -> Vec<Diagnostic> {
        whitespace::check_whitespace_uniformity(ctx.source, "ts-whitespace-uniformity", 50)
    }
}
