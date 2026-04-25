use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags functions whose body is just a single expression delegating to another call.
///
/// AI loves generating wrappers like:
/// ```ignore
/// fn get_name(&self) -> String {
///     self.name.clone()
/// }
/// fn process(data: &[u8]) -> Result<(), Error> {
///     internal_process(data)
/// }
/// ```
///
/// One or two of these is fine (newtype patterns, trait impls), but a file
/// full of trivial delegators is a slop signal. Files whose names suggest
/// API surface (runtime.rs, types.rs, etc.) get a higher threshold.
pub struct TrivialWrapper;

/// Files where thin wrappers are expected API design, not slop.
const API_SURFACE_FILES: &[&str] = &[
    "runtime", "types", "config", "constants", "consts",
    "paths", "defaults", "prelude", "helpers", "api",
];

impl Rule for TrivialWrapper {
    fn name(&self) -> &'static str {
        "trivial-wrapper"
    }

    fn check(&self, file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = WrapperVisitor { hits: Vec::new() };
        visitor.visit_file(file);

        let threshold = if is_api_surface_file(ctx.filename) { 10 } else { 5 };

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

    // Deliberately skip impl methods — single-expression bodies are normal
    // for trait impls (Default::default, Display::fmt, etc.).
}
