pub mod accessibility;
pub mod css_smells;
pub mod div_soup;
pub mod generic_classes;
pub mod inline_styles;
pub mod missing_semantics;
pub mod parse;

use crate::diagnostic::Diagnostic;

/// Pre-parsed HTML context shared across all rules.
pub struct HtmlContext<'a> {
    pub filename: &'a str,
    pub source: &'a str,
    pub parsed: parse::ParsedHtml,
}

impl<'a> HtmlContext<'a> {
    pub fn new(filename: &'a str, source: &'a str) -> Self {
        let parsed = parse::extract_tags(source);
        Self { filename, source, parsed }
    }
}

/// Trait that all HTML/CSS lint rules implement.
pub trait HtmlRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, ctx: &HtmlContext) -> Vec<Diagnostic>;
}
