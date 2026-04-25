use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags functions whose body is just a single expression delegating to another call.
///
/// One or two of these is fine (newtype patterns, trait impls), but a file
/// full of trivial delegators is a slop signal. Files whose names suggest
/// API surface or orchestration get higher thresholds.
pub struct TrivialWrapper;

/// Files where thin wrappers are expected API design.
const API_SURFACE_FILES: &[&str] = &[
    "runtime", "types", "config", "constants", "consts",
    "paths", "defaults", "prelude", "helpers", "api",
];

/// Files with orchestration/collector patterns where many small helpers
/// that each extract one check are the correct architecture.
const ORCHESTRATION_PATTERNS: &[&str] = &[
    "ast", "collect", "check", "parse", "extract", "analyze",
    "kubernetes", "ci", "best_practices",
];

impl Rule for TrivialWrapper {
    fn name(&self) -> &'static str {
        "trivial-wrapper"
    }

    fn check(&self, file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = WrapperVisitor { hits: Vec::new() };
        visitor.visit_file(file);

        let threshold = if is_api_surface_file(ctx.filename) || is_orchestration_file(ctx.filename) {
            15
        } else {
            6
        };

        if visitor.hits.len() < threshold {
            visitor.hits.clear();
        }

        visitor.hits
    }
}

fn is_api_surface_file(filename: &str) -> bool {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    API_SURFACE_FILES.contains(&stem)
}

fn is_orchestration_file(filename: &str) -> bool {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    ORCHESTRATION_PATTERNS.iter().any(|p| stem.contains(p))
}

struct WrapperVisitor {
    hits: Vec<Diagnostic>,
}

fn is_single_expr_body(block: &syn::Block) -> bool {
    matches!(block.stmts.as_slice(), [syn::Stmt::Expr(_, _)])
}

impl<'ast> Visit<'ast> for WrapperVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if is_single_expr_body(&node.block) {
            let line = node.sig.fn_token.span.start().line;
            let name = node.sig.ident.to_string();
            self.hits.push(Diagnostic {
                rule: "trivial-wrapper",
                message: format!("`fn {name}` is a single-expression wrapper — does it add value?"),
                line,
                severity: Severity::Hint,
                weight: 0.75,
            });
        }
        syn::visit::visit_item_fn(self, node);
    }

    // Skip impl methods — single-expression bodies are normal for trait impls.
}
