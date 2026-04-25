pub mod restating_comments;
pub mod generic_naming;
pub mod bare_catch;

use crate::diagnostic::Diagnostic;

/// Context for Java analysis.
///
/// Java support is minimal — enough to catch obvious slop in legacy
/// codebases being maintained with AI assistance. If you're starting
/// a new project in Java in 2026, lipstyk is the least of your problems.
pub struct JavaContext<'a> {
    pub filename: &'a str,
    pub source: &'a str,
}

pub trait JavaRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, ctx: &JavaContext) -> Vec<Diagnostic>;
}
