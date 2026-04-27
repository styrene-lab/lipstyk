use crate::diagnostic::Diagnostic;
use crate::golang::ast::GoParsed;
use crate::html::parse::ParsedHtml;
use crate::oxc::OxcParsed;

/// Language/file-type identifier for dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    TypeScript,
    JavaScript,
    Python,
    Html,
    Css,
    Java,
    Go,
    Shell,
    Dockerfile,
    Yaml,
    Markdown,
    Text,
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
            "go" => Some(Self::Go),
            "sh" | "bash" | "zsh" => Some(Self::Shell),
            "yml" | "yaml" => Some(Self::Yaml),
            "md" | "mdx" => Some(Self::Markdown),
            "txt" | "text" | "email" => Some(Self::Text),
            _ => None,
        }
    }

    pub fn from_filename(filename: &str) -> Option<Self> {
        let name = std::path::Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(filename);

        if let Some(ext) = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            && let Some(lang) = Self::from_ext(ext)
        {
            return Some(lang);
        }

        match name {
            "Dockerfile" | "Containerfile" => Some(Self::Dockerfile),
            "Makefile" | "makefile" => Some(Self::Shell),
            _ => None,
        }
    }
}

/// Context for all non-Rust rules.
///
/// Carries source text, language, and optional pre-parsed data:
/// - `html`: tag-parsed HTML (for HTML/CSS files)
/// - `oxc`: typed AST (for TS/JS files, via oxc_parser)
pub struct SourceContext<'a> {
    pub filename: &'a str,
    pub source: &'a str,
    pub lang: Lang,
    pub html: Option<ParsedHtml>,
    pub oxc: Option<OxcParsed>,
    pub go: Option<GoParsed>,
}

impl<'a> SourceContext<'a> {
    pub fn new(filename: &'a str, source: &'a str, lang: Lang) -> Self {
        let html = match lang {
            Lang::Html | Lang::Css => Some(crate::html::parse::extract_tags(source)),
            _ => None,
        };
        let oxc = match lang {
            Lang::TypeScript | Lang::JavaScript => crate::oxc::parse_ts(source, filename),
            _ => None,
        };
        let go = match lang {
            Lang::Go => crate::golang::ast::parse_go(source),
            _ => None,
        };
        Self {
            filename,
            source,
            lang,
            html,
            oxc,
            go,
        }
    }

    pub fn is_ts_or_js(&self) -> bool {
        matches!(self.lang, Lang::TypeScript | Lang::JavaScript)
    }

    pub fn is_ts_only(&self) -> bool {
        matches!(self.lang, Lang::TypeScript)
    }
}

/// Unified trait for all non-Rust lint rules.
pub trait SourceRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn langs(&self) -> &[Lang];
    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic>;
}
