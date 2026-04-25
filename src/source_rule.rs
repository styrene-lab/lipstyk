use crate::diagnostic::Diagnostic;
use crate::html::parse::ParsedHtml;

/// Language identifier for dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    TypeScript,
    JavaScript,
    Python,
    Html,
    Css,
    Java,
}

impl Lang {
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "ts" | "tsx" => Some(Self::TypeScript),
            "js" | "jsx" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "html" | "htm" | "vue" | "svelte" => Some(Self::Html),
            "css" => Some(Self::Css),
            "java" => Some(Self::Java),
            _ => None,
        }
    }
}

/// Context for all non-Rust (text-based) rules.
///
/// Carries the source text, filename, detected language, and
/// pre-parsed HTML tags (populated only for HTML/CSS files).
pub struct SourceContext<'a> {
    pub filename: &'a str,
    pub source: &'a str,
    pub lang: Lang,
    pub html: Option<ParsedHtml>,
}

impl<'a> SourceContext<'a> {
    pub fn new(filename: &'a str, source: &'a str, lang: Lang) -> Self {
        let html = match lang {
            Lang::Html | Lang::Css => Some(crate::html::parse::extract_tags(source)),
            _ => None,
        };
        Self { filename, source, lang, html }
    }

    pub fn is_ts_or_js(&self) -> bool {
        matches!(self.lang, Lang::TypeScript | Lang::JavaScript)
    }

    pub fn is_ts_only(&self) -> bool {
        matches!(self.lang, Lang::TypeScript)
    }
}

/// Unified trait for all non-Rust lint rules.
///
/// Each rule declares which languages it applies to. The linter skips
/// rules whose language doesn't match the file being analyzed.
pub trait SourceRule: Send + Sync {
    fn name(&self) -> &'static str;

    /// Which languages this rule applies to.
    fn langs(&self) -> &[Lang];

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic>;
}
