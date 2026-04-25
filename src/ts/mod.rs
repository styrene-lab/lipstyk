pub mod any_abuse;
pub mod console_dump;
pub mod generic_naming;
pub mod nested_ternary;
pub mod promise_antipattern;
pub mod restating_comments;
pub mod whitespace_uniformity;

use crate::diagnostic::Diagnostic;

/// Context for TypeScript/JavaScript analysis.
pub struct TsContext<'a> {
    pub filename: &'a str,
    pub source: &'a str,
}

/// Trait for TypeScript/JavaScript lint rules.
///
/// These operate on source text without a full TS parser. Covers
/// `.ts`, `.tsx`, `.js`, `.jsx` files.
pub trait TsRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, ctx: &TsContext) -> Vec<Diagnostic>;
}
