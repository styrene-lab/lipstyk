use crate::diagnostic::Diagnostic;
use crate::html::parse::ParsedHtml;

/// Language/file-type identifier for dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    TypeScript,
    JavaScript,
    Python,
    Html,
    Css,
    Java,
    Shell,
    Dockerfile,
    Yaml,
    Markdown,
}

impl Lang {
    /// Detect language from file extension.
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "ts" | "tsx" => Some(Self::TypeScript),
            "js" | "jsx" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "html" | "htm" | "vue" | "svelte" => Some(Self::Html),
            "css" => Some(Self::Css),
            "java" => Some(Self::Java),
            "sh" | "bash" | "zsh" => Some(Self::Shell),
            "yml" | "yaml" => Some(Self::Yaml),
            "md" | "mdx" => Some(Self::Markdown),
            _ => None,
        }
    }

    /// Detect from filename (not just extension) for extensionless files.
    pub fn from_filename(filename: &str) -> Option<Self> {
        let name = std::path::Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(filename);

        // Try extension first.
        if let Some(ext) = std::path::Path::new(filename).extension().and_then(|e| e.to_str())
            && let Some(lang) = Self::from_ext(ext) {
                return Some(lang);
            }

        // Extensionless filename detection.
        match name {
            "Dockerfile" | "Containerfile" => Some(Self::Dockerfile),
            "Makefile" | "makefile" => Some(Self::Shell), // close enough
            _ => None,
        }
    }
}

/// Context for all non-Rust (text-based) rules.
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
pub trait SourceRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn langs(&self) -> &[Lang];
    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic>;
}
