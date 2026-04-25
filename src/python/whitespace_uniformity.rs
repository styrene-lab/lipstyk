use crate::common::whitespace;
use crate::diagnostic::Diagnostic;
use crate::python::{PyContext, PyRule};

pub struct WhitespaceUniformity;

impl PyRule for WhitespaceUniformity {
    fn name(&self) -> &'static str {
        "py-whitespace-uniformity"
    }

    fn check(&self, ctx: &PyContext) -> Vec<Diagnostic> {
        whitespace::check_whitespace_uniformity(ctx.source, "py-whitespace-uniformity", 50)
    }
}
