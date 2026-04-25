use std::collections::HashSet;

use crate::config::Config;
use crate::diagnostic::SlopScore;
use crate::diff;
use crate::html::{HtmlContext, HtmlRule};
use crate::java::{JavaContext, JavaRule};
use crate::python::{PyContext, PyRule};
use crate::rules::{LintContext, Rule};
use crate::ts::{TsContext, TsRule};
use tracing::{debug, info, instrument, warn};

/// The main entry point for library consumers.
/// Dispatches to language-specific rule sets based on file extension.
pub struct Linter {
    rust_rules: Vec<Box<dyn Rule>>,
    html_rules: Vec<Box<dyn HtmlRule>>,
    ts_rules: Vec<Box<dyn TsRule>>,
    py_rules: Vec<Box<dyn PyRule>>,
    java_rules: Vec<Box<dyn JavaRule>>,
    exclude_tests: bool,
    config: Config,
    /// If set, only report diagnostics on these lines (diff mode).
    changed_lines: Option<std::collections::HashMap<String, HashSet<usize>>>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rust_rules: Vec::new(),
            html_rules: Vec::new(),
            ts_rules: Vec::new(),
            py_rules: Vec::new(),
            java_rules: Vec::new(),
            exclude_tests: false,
            config: Config::default(),
            changed_lines: None,
        }
    }

    /// Create a linter with all built-in rules for all supported languages.
    pub fn with_defaults() -> Self {
        let mut linter = Self::new();

        // Rust rules (21)
        linter.add_rust_rule(Box::new(crate::rules::unwrap_overuse::UnwrapOveruse));
        linter.add_rust_rule(Box::new(crate::rules::redundant_clone::RedundantClone));
        linter.add_rust_rule(Box::new(crate::rules::restating_comments::RestatingComments));
        linter.add_rust_rule(Box::new(crate::rules::needless_type_annotation::NeedlessTypeAnnotation));
        linter.add_rust_rule(Box::new(crate::rules::trivial_wrapper::TrivialWrapper));
        linter.add_rust_rule(Box::new(crate::rules::verbose_match::VerboseMatch));
        linter.add_rust_rule(Box::new(crate::rules::generic_naming::GenericNaming));
        linter.add_rust_rule(Box::new(crate::rules::over_documentation::OverDocumentation));
        linter.add_rust_rule(Box::new(crate::rules::string_params::StringParams));
        linter.add_rust_rule(Box::new(crate::rules::pub_overuse::PubOveruse));
        linter.add_rust_rule(Box::new(crate::rules::index_loop::IndexLoop));
        linter.add_rust_rule(Box::new(crate::rules::generic_todo::GenericTodo));
        linter.add_rust_rule(Box::new(crate::rules::error_swallowing::ErrorSwallowing));
        linter.add_rust_rule(Box::new(crate::rules::whitespace_uniformity::WhitespaceUniformity));
        linter.add_rust_rule(Box::new(crate::rules::structural_repetition::StructuralRepetition));
        linter.add_rust_rule(Box::new(crate::rules::needless_lifetimes::NeedlessLifetimes));
        linter.add_rust_rule(Box::new(crate::rules::boxed_error::BoxedError));
        linter.add_rust_rule(Box::new(crate::rules::derive_stacking::DeriveStacking));
        linter.add_rust_rule(Box::new(crate::rules::dead_code_markers::DeadCodeMarkers));
        linter.add_rust_rule(Box::new(crate::rules::comment_clustering::CommentClustering));
        linter.add_rust_rule(Box::new(crate::rules::naming_entropy::NamingEntropy));

        // HTML/CSS rules (6)
        linter.add_html_rule(Box::new(crate::html::div_soup::DivSoup));
        linter.add_html_rule(Box::new(crate::html::missing_semantics::MissingSemantics));
        linter.add_html_rule(Box::new(crate::html::inline_styles::InlineStyles));
        linter.add_html_rule(Box::new(crate::html::generic_classes::GenericClasses));
        linter.add_html_rule(Box::new(crate::html::accessibility::Accessibility));
        linter.add_html_rule(Box::new(crate::html::css_smells::CssSmells));

        // TypeScript/JavaScript rules (7)
        linter.add_ts_rule(Box::new(crate::ts::any_abuse::AnyAbuse));
        linter.add_ts_rule(Box::new(crate::ts::console_dump::ConsoleDump));
        linter.add_ts_rule(Box::new(crate::ts::nested_ternary::NestedTernary));
        linter.add_ts_rule(Box::new(crate::ts::promise_antipattern::PromiseAntipattern));
        linter.add_ts_rule(Box::new(crate::ts::generic_naming::GenericNaming));
        linter.add_ts_rule(Box::new(crate::ts::restating_comments::RestatingComments));
        linter.add_ts_rule(Box::new(crate::ts::whitespace_uniformity::WhitespaceUniformity));

        // Python rules (7)
        linter.add_py_rule(Box::new(crate::python::bare_except::BareExcept));
        linter.add_py_rule(Box::new(crate::python::print_debug::PrintDebug));
        linter.add_py_rule(Box::new(crate::python::import_star::ImportStar));
        linter.add_py_rule(Box::new(crate::python::type_hint_gaps::TypeHintGaps));
        linter.add_py_rule(Box::new(crate::python::generic_naming::GenericNaming));
        linter.add_py_rule(Box::new(crate::python::restating_comments::RestatingComments));
        linter.add_py_rule(Box::new(crate::python::whitespace_uniformity::WhitespaceUniformity));

        // Java rules (3) — legacy language, minimal coverage
        linter.add_java_rule(Box::new(crate::java::restating_comments::RestatingComments));
        linter.add_java_rule(Box::new(crate::java::generic_naming::GenericNaming));
        linter.add_java_rule(Box::new(crate::java::bare_catch::BareCatch));

        info!(
            rust = linter.rust_rules.len(),
            html = linter.html_rules.len(),
            ts = linter.ts_rules.len(),
            py = linter.py_rules.len(),
            "linter initialized"
        );
        linter
    }

    pub fn exclude_tests(mut self, yes: bool) -> Self {
        self.exclude_tests = yes;
        self
    }

    /// Apply a project config (rule enable/disable, weight overrides).
    pub fn with_config(mut self, config: Config) -> Self {
        self.exclude_tests = config.settings.exclude_tests || self.exclude_tests;
        self.config = config;
        self
    }

    /// Enable diff mode — only report diagnostics on changed lines.
    pub fn with_diff(mut self, base: Option<&str>) -> Self {
        self.changed_lines = Some(diff::changed_lines_from_git(base));
        self
    }

    pub fn add_rust_rule(&mut self, rule: Box<dyn Rule>) {
        self.rust_rules.push(rule);
    }

    pub fn add_html_rule(&mut self, rule: Box<dyn HtmlRule>) {
        self.html_rules.push(rule);
    }

    pub fn add_ts_rule(&mut self, rule: Box<dyn TsRule>) {
        self.ts_rules.push(rule);
    }

    pub fn add_py_rule(&mut self, rule: Box<dyn PyRule>) {
        self.py_rules.push(rule);
    }

    pub fn add_java_rule(&mut self, rule: Box<dyn JavaRule>) {
        self.java_rules.push(rule);
    }

    #[instrument(skip(self, source), fields(diagnostics, score))]
    pub fn lint_source(&self, filename: &str, source: &str) -> Result<SlopScore, crate::LintError> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let mut diagnostics = match ext {
            "rs" => self.lint_rust(filename, source)?,
            "html" | "htm" | "vue" | "svelte" | "css" => self.lint_html(filename, source),
            "ts" | "tsx" | "js" | "jsx" => self.lint_ts(filename, source),
            "py" => self.lint_py(filename, source),
            "java" => self.lint_java(filename, source),
            _ => Vec::new(),
        };

        // Filter disabled rules.
        diagnostics.retain(|d| self.config.is_rule_enabled(d.rule));

        // Apply weight overrides.
        for d in &mut diagnostics {
            if let Some(w) = self.config.weight_override(d.rule) {
                d.weight = w;
            }
        }

        // Apply diff filter if in diff mode.
        if let Some(ref changed) = self.changed_lines {
            let file_changes = changed.iter().find(|(path, _)| {
                diff::normalize_diff_path(path, filename)
            });

            if let Some((_, lines)) = file_changes {
                diff::filter_to_changed(&mut diagnostics, lines, 3);
            } else {
                // File not in diff — skip entirely.
                diagnostics.clear();
            }
        }

        let score = SlopScore::new(filename.to_string(), diagnostics);

        tracing::Span::current().record("diagnostics", score.diagnostics.len());
        tracing::Span::current().record("score", score.total);

        if !score.diagnostics.is_empty() {
            info!(
                file = filename,
                score = score.total,
                diagnostics = score.diagnostics.len(),
                "file analyzed"
            );
        }

        Ok(score)
    }

    fn lint_rust(&self, filename: &str, source: &str) -> Result<Vec<crate::Diagnostic>, crate::LintError> {
        let syntax = syn::parse_file(source).map_err(|e| {
            warn!(file = filename, error = %e, "parse failed");
            crate::LintError::Parse {
                file: filename.to_string(),
                message: e.to_string(),
            }
        })?;

        let ctx = LintContext {
            filename,
            source,
            exclude_tests: self.exclude_tests,
        };

        let mut diagnostics = Vec::new();
        for rule in &self.rust_rules {
            if !self.config.is_rule_enabled(rule.name()) {
                continue;
            }
            let before = diagnostics.len();
            diagnostics.extend(rule.check(&syntax, &ctx));
            let found = diagnostics.len() - before;
            if found > 0 {
                debug!(rule = rule.name(), findings = found, "rule fired");
            }
        }

        diagnostics.sort_by_key(|d| d.line);
        Ok(diagnostics)
    }

    fn lint_html(&self, filename: &str, source: &str) -> Vec<crate::Diagnostic> {
        let ctx = HtmlContext::new(filename, source);
        run_rules(&self.html_rules, &self.config, |rule| rule.check(&ctx), |rule| rule.name())
    }

    fn lint_ts(&self, filename: &str, source: &str) -> Vec<crate::Diagnostic> {
        let ctx = TsContext { filename, source };
        run_rules(&self.ts_rules, &self.config, |rule| rule.check(&ctx), |rule| rule.name())
    }

    fn lint_py(&self, filename: &str, source: &str) -> Vec<crate::Diagnostic> {
        let ctx = PyContext { filename, source };
        run_rules(&self.py_rules, &self.config, |rule| rule.check(&ctx), |rule| rule.name())
    }

    fn lint_java(&self, filename: &str, source: &str) -> Vec<crate::Diagnostic> {
        let ctx = JavaContext { filename, source };
        run_rules(&self.java_rules, &self.config, |rule| rule.check(&ctx), |rule| rule.name())
    }
}

fn run_rules<R: ?Sized>(
    rules: &[Box<R>],
    config: &Config,
    checker: impl Fn(&R) -> Vec<crate::Diagnostic>,
    namer: impl Fn(&R) -> &str,
) -> Vec<crate::Diagnostic> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        if !config.is_rule_enabled(namer(rule)) {
            continue;
        }
        let found = checker(rule);
        if !found.is_empty() {
            debug!(rule = namer(rule), findings = found.len(), "rule fired");
            diagnostics.extend(found);
        }
    }
    diagnostics.sort_by_key(|d| d.line);
    diagnostics
}

impl Default for Linter {
    fn default() -> Self {
        Self::with_defaults()
    }
}
