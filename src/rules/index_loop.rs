use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags C-style index loops: `for i in 0..vec.len() { vec[i] }`.
///
/// AI code generators default to index-based iteration instead of
/// idiomatic `.iter()`, `.iter_mut()`, or `.enumerate()`. This is
/// one of the most recognizable AI patterns in Rust.
pub struct IndexLoop;

impl Rule for IndexLoop {
    fn name(&self) -> &'static str {
        "index-loop"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = LoopVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct LoopVisitor {
    hits: Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for LoopVisitor {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        // Look for `for i in 0..something.len()`
        if let syn::Expr::Range(range) = node.expr.as_ref() {
            let starts_at_zero = range.start.as_ref().is_some_and(|s| is_zero_literal(s));
            let ends_at_len = range.end.as_ref().is_some_and(|e| is_len_call(e));

            if starts_at_zero && ends_at_len {
                let line = node.for_token.span.start().line;
                self.hits.push(Diagnostic {
                    rule: "index-loop",
                    message: "C-style index loop — consider `.iter()` or `.enumerate()`"
                        .to_string(),
                    line,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        syn::visit::visit_expr_for_loop(self, node);
    }
}

fn is_zero_literal(expr: &syn::Expr) -> bool {
    if let syn::Expr::Lit(lit) = expr
        && let syn::Lit::Int(int_lit) = &lit.lit
    {
        return int_lit.base10_digits() == "0";
    }
    false
}

fn is_len_call(expr: &syn::Expr) -> bool {
    if let syn::Expr::MethodCall(call) = expr {
        return call.method == "len" && call.args.is_empty();
    }
    false
}
