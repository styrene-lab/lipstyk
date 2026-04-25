pub mod bare_except;
pub mod generic_naming;
pub mod import_star;
pub mod print_debug;
pub mod restating_comments;
pub mod type_hint_gaps;
pub mod whitespace_uniformity;

use crate::diagnostic::Diagnostic;

/// Context for Python analysis.
pub struct PyContext<'a> {
    pub filename: &'a str,
    pub source: &'a str,
}

/// Trait for Python lint rules.
pub trait PyRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, ctx: &PyContext) -> Vec<Diagnostic>;
}
