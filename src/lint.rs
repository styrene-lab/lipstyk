use std::collections::HashSet;

use crate::config::Config;
use crate::diagnostic::SlopScore;
use crate::diff;
use crate::rules::{LintContext, Rule};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use tracing::{debug, info, instrument, warn};

/// The main entry point for library consumers.
///
/// Holds Rust rules (AST-based, need `syn`) and source rules
/// (text-based, unified across all other languages).
pub struct Linter {
    rust_rules: Vec<Box<dyn Rule>>,
    source_rules: Vec<Box<dyn SourceRule>>,
    exclude_tests: bool,
    config: Config,
    changed_lines: Option<std::collections::HashMap<String, HashSet<usize>>>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rust_rules: Vec::new(),
            source_rules: Vec::new(),
            exclude_tests: false,
            config: Config::default(),
            changed_lines: None,
        }
    }

    /// Create a linter with all built-in rules for all supported languages.
    pub fn with_defaults() -> Self {
        let mut linter = Self::new();

        // Rust rules — AST-level via syn
        linter.add_rust_rule(Box::new(crate::rules::unwrap_overuse::UnwrapOveruse));
        linter.add_rust_rule(Box::new(crate::rules::redundant_clone::RedundantClone));
        linter.add_rust_rule(Box::new(
            crate::rules::restating_comments::RestatingComments,
        ));
        linter.add_rust_rule(Box::new(
            crate::rules::needless_type_annotation::NeedlessTypeAnnotation,
        ));
        linter.add_rust_rule(Box::new(crate::rules::trivial_wrapper::TrivialWrapper));
        linter.add_rust_rule(Box::new(crate::rules::verbose_match::VerboseMatch));
        linter.add_rust_rule(Box::new(crate::rules::generic_naming::GenericNaming));
        linter.add_rust_rule(Box::new(
            crate::rules::over_documentation::OverDocumentation,
        ));
        linter.add_rust_rule(Box::new(crate::rules::string_params::StringParams));
        linter.add_rust_rule(Box::new(crate::rules::pub_overuse::PubOveruse));
        linter.add_rust_rule(Box::new(crate::rules::index_loop::IndexLoop));
        linter.add_rust_rule(Box::new(crate::rules::generic_todo::GenericTodo));
        linter.add_rust_rule(Box::new(crate::rules::error_swallowing::ErrorSwallowing));
        linter.add_rust_rule(Box::new(
            crate::rules::whitespace_uniformity::WhitespaceUniformity,
        ));
        linter.add_rust_rule(Box::new(
            crate::rules::structural_repetition::StructuralRepetition,
        ));
        linter.add_rust_rule(Box::new(
            crate::rules::needless_lifetimes::NeedlessLifetimes,
        ));
        linter.add_rust_rule(Box::new(crate::rules::boxed_error::BoxedError));
        linter.add_rust_rule(Box::new(crate::rules::derive_stacking::DeriveStacking));
        linter.add_rust_rule(Box::new(crate::rules::dead_code_markers::DeadCodeMarkers));
        linter.add_rust_rule(Box::new(
            crate::rules::comment_clustering::CommentClustering,
        ));
        linter.add_rust_rule(Box::new(crate::rules::naming_entropy::NamingEntropy));

        // Source rules — text-based, all other languages
        linter.add_source_rule(Box::new(crate::html::div_soup::DivSoup));
        linter.add_source_rule(Box::new(crate::html::missing_semantics::MissingSemantics));
        linter.add_source_rule(Box::new(crate::html::inline_styles::InlineStyles));
        linter.add_source_rule(Box::new(crate::html::generic_classes::GenericClasses));
        linter.add_source_rule(Box::new(crate::html::accessibility::Accessibility));
        linter.add_source_rule(Box::new(crate::html::css_smells::CssSmells));
        linter.add_source_rule(Box::new(crate::ts::any_abuse::AnyAbuse));
        linter.add_source_rule(Box::new(crate::ts::console_dump::ConsoleDump));
        linter.add_source_rule(Box::new(crate::ts::fixed_delay_sync::FixedDelaySync));
        linter.add_source_rule(Box::new(crate::ts::nested_ternary::NestedTernary));
        linter.add_source_rule(Box::new(crate::ts::promise_antipattern::PromiseAntipattern));
        linter.add_source_rule(Box::new(crate::ts::generic_naming::GenericNaming));
        linter.add_source_rule(Box::new(crate::ts::restating_comments::RestatingComments));
        linter.add_source_rule(Box::new(
            crate::ts::whitespace_uniformity::WhitespaceUniformity,
        ));
        linter.add_source_rule(Box::new(crate::ts::error_handling::ErrorHandling));
        linter.add_source_rule(Box::new(crate::ts::comment_depth::CommentDepth));
        linter.add_source_rule(Box::new(
            crate::ts::structural_repetition::StructuralRepetition,
        ));
        linter.add_source_rule(Box::new(
            crate::ts::structural_repetition::PyStructuralRepetition,
        ));
        linter.add_source_rule(Box::new(crate::ts::naming_entropy::NamingEntropy));
        linter.add_source_rule(Box::new(crate::ts::naming_entropy::PyNamingEntropy));
        linter.add_source_rule(Box::new(crate::ts::trivial_wrapper::TrivialWrapper));
        linter.add_source_rule(Box::new(crate::ts::redundant_async::RedundantAsync));
        linter.add_source_rule(Box::new(crate::ts::nesting_depth::NestingDepth));
        linter.add_source_rule(Box::new(crate::ts::nesting_depth::PyNestingDepth));
        linter.add_source_rule(Box::new(crate::python::bare_except::BareExcept));
        linter.add_source_rule(Box::new(crate::python::print_debug::PrintDebug));
        linter.add_source_rule(Box::new(crate::python::import_star::ImportStar));
        linter.add_source_rule(Box::new(crate::python::type_hint_gaps::TypeHintGaps));
        linter.add_source_rule(Box::new(crate::python::generic_naming::GenericNaming));
        linter.add_source_rule(Box::new(
            crate::python::restating_comments::RestatingComments,
        ));
        linter.add_source_rule(Box::new(
            crate::python::whitespace_uniformity::WhitespaceUniformity,
        ));
        linter.add_source_rule(Box::new(crate::python::error_handling::ErrorHandling));
        linter.add_source_rule(Box::new(crate::python::comment_depth::CommentDepth));
        linter.add_source_rule(Box::new(crate::python::index_loop::IndexLoop));
        linter.add_source_rule(Box::new(crate::python::mutable_default::MutableDefault));
        linter.add_source_rule(Box::new(crate::python::trivial_wrapper::TrivialWrapper));
        linter.add_source_rule(Box::new(crate::java::restating_comments::RestatingComments));
        linter.add_source_rule(Box::new(crate::java::generic_naming::GenericNaming));
        linter.add_source_rule(Box::new(crate::java::bare_catch::BareCatch));
        linter.add_source_rule(Box::new(crate::java::comment_depth::CommentDepth));

        // Go rules
        linter.add_source_rule(Box::new(crate::golang::error_handling::ErrorHandling));
        linter.add_source_rule(Box::new(crate::golang::antipatterns::Antipatterns));
        linter.add_source_rule(Box::new(crate::golang::generic_naming::GenericNaming));
        linter.add_source_rule(Box::new(
            crate::golang::restating_comments::RestatingComments,
        ));
        linter.add_source_rule(Box::new(crate::golang::comment_depth::CommentDepth));
        linter.add_source_rule(Box::new(
            crate::golang::structural_repetition::StructuralRepetition,
        ));
        linter.add_source_rule(Box::new(crate::golang::naming_entropy::NamingEntropy));
        linter.add_source_rule(Box::new(crate::golang::nesting_depth::NestingDepth));

        // Shell rules
        linter.add_source_rule(Box::new(crate::shell::strict_mode::StrictMode));
        linter.add_source_rule(Box::new(crate::shell::quoting::Quoting));
        linter.add_source_rule(Box::new(crate::shell::antipatterns::Antipatterns));

        // Dockerfile rules
        linter.add_source_rule(Box::new(crate::docker::best_practices::BestPractices));

        // Markdown rules
        linter.add_source_rule(Box::new(crate::prose::SlopPhrases));
        linter.add_source_rule(Box::new(crate::prose::Structure));
        linter.add_source_rule(Box::new(crate::markdown::slop_phrases::SlopPhrases));
        linter.add_source_rule(Box::new(crate::markdown::structure::Structure));
        linter.add_source_rule(Box::new(crate::markdown::placeholders::Placeholders));

        // DevOps YAML rules (content-sniffed)
        linter.add_source_rule(Box::new(crate::devops::kubernetes::KubernetesRules));
        linter.add_source_rule(Box::new(crate::devops::ci::CiRules));

        info!(
            rust = linter.rust_rules.len(),
            source = linter.source_rules.len(),
            "linter initialized"
        );
        linter
    }

    /// Names of all registered built-in rules.
    pub fn rule_names(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self
            .rust_rules
            .iter()
            .map(|rule| rule.name())
            .chain(self.source_rules.iter().map(|rule| rule.name()))
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    pub fn rule_counts(&self) -> RuleCounts {
        let (mut html, mut css, mut ts, mut js, mut py, mut java, mut go) = (0, 0, 0, 0, 0, 0, 0);
        let (mut shell, mut docker, mut yaml, mut markdown, mut text) = (0, 0, 0, 0, 0);

        for rule in &self.source_rules {
            for lang in rule.langs() {
                match lang {
                    Lang::Html => html += 1,
                    Lang::Css => css += 1,
                    Lang::TypeScript => ts += 1,
                    Lang::JavaScript => js += 1,
                    Lang::Python => py += 1,
                    Lang::Java => java += 1,
                    Lang::Go => go += 1,
                    Lang::Shell => shell += 1,
                    Lang::Dockerfile => docker += 1,
                    Lang::Yaml => yaml += 1,
                    Lang::Markdown => markdown += 1,
                    Lang::Text => text += 1,
                }
            }
        }

        RuleCounts {
            rust: self.rust_rules.len(),
            html,
            css,
            ts,
            js,
            py,
            java,
            go,
            shell,
            docker,
            yaml,
            markdown,
            text,
        }
    }

    pub fn exclude_tests(mut self, yes: bool) -> Self {
        self.exclude_tests = yes;
        self
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.exclude_tests = config.settings.exclude_tests || self.exclude_tests;
        self.config = config;
        self
    }

    pub fn with_diff(mut self, base: Option<&str>) -> Self {
        self.changed_lines = Some(diff::changed_lines_from_git(base));
        self
    }

    pub fn add_rust_rule(&mut self, rule: Box<dyn Rule>) {
        self.rust_rules.push(rule);
    }

    pub fn add_source_rule(&mut self, rule: Box<dyn SourceRule>) {
        self.source_rules.push(rule);
    }

    /// Run cross-file analysis after all per-file linting is complete.
    ///
    /// Takes the per-file scores and source texts, detects codebase-level
    /// patterns (duplicated blocks, identical imports, cloned error handling),
    /// and merges additional diagnostics into the affected file scores.
    pub fn lint_codebase(
        &self,
        scores: &mut [SlopScore],
        sources: &std::collections::BTreeMap<String, String>,
    ) {
        let extra = crate::cross_file::analyze(scores, sources);

        for (filename, diagnostics) in extra {
            // Find or create the score entry for this file.
            if let Some(score) = scores.iter_mut().find(|s| s.file == filename) {
                for d in diagnostics {
                    if self.config.is_rule_enabled(d.rule) {
                        score.total += d.weight;
                        score.diagnostics.push(d);
                    }
                }
            }
        }
    }

    #[instrument(skip(self, source), fields(diagnostics, score))]
    pub fn lint_source(&self, filename: &str, source: &str) -> Result<SlopScore, crate::LintError> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let mut diagnostics = if ext == "rs" {
            self.lint_rust(filename, source)?
        } else if let Some(lang) = Lang::from_filename(filename) {
            self.lint_source_rules(filename, source, lang)
        } else {
            Vec::new()
        };

        diagnostics.retain(|d| self.config.is_rule_enabled(d.rule));

        for d in &mut diagnostics {
            if let Some(w) = self.config.weight_override(d.rule) {
                d.weight = w;
            }
        }

        if let Some(ref changed) = self.changed_lines {
            let file_changes = changed
                .iter()
                .find(|(path, _)| diff::normalize_diff_path(path, filename));
            if let Some((_, lines)) = file_changes {
                diff::filter_to_changed(&mut diagnostics, lines, 3);
            } else {
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

    fn lint_rust(
        &self,
        filename: &str,
        source: &str,
    ) -> Result<Vec<crate::Diagnostic>, crate::LintError> {
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

    fn lint_source_rules(
        &self,
        filename: &str,
        source: &str,
        lang: Lang,
    ) -> Vec<crate::Diagnostic> {
        let ctx = SourceContext::new(filename, source, lang);

        let mut diagnostics = Vec::new();
        for rule in &self.source_rules {
            if !rule.langs().contains(&lang) {
                continue;
            }
            if !self.config.is_rule_enabled(rule.name()) {
                continue;
            }
            let before = diagnostics.len();
            diagnostics.extend(rule.check(&ctx));
            let found = diagnostics.len() - before;
            if found > 0 {
                debug!(rule = rule.name(), findings = found, "rule fired");
            }
        }

        diagnostics.sort_by_key(|d| d.line);
        diagnostics
    }
}

#[derive(Default)]
pub struct RuleCounts {
    pub rust: usize,
    pub html: usize,
    pub css: usize,
    pub ts: usize,
    pub js: usize,
    pub py: usize,
    pub java: usize,
    pub go: usize,
    pub shell: usize,
    pub docker: usize,
    pub yaml: usize,
    pub markdown: usize,
    pub text: usize,
}

impl RuleCounts {
    pub fn total(&self) -> usize {
        self.rust + self.source_total()
    }

    pub fn source_total(&self) -> usize {
        self.html.max(self.css)
            + self.ts.max(self.js)
            + self.py
            + self.java
            + self.go
            + self.shell
            + self.docker
            + self.yaml
            + self.markdown
            + self.text
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::with_defaults()
    }
}
