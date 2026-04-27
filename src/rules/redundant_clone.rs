use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags `.clone()` calls that look gratuitous.
///
/// AI scatters `.clone()` to satisfy the borrow checker instead of
/// restructuring ownership. Density drives the score.
///
/// Suppressed:
/// - Inside closures on field access (extracting from a borrow)
/// - On method call receivers (`.clone()` on the return of another call
///   is often a framework pattern, not gratuitous)
/// - Inside async fn bodies (tower-lsp, axum, actix patterns mandate clones
///   to move values into futures)
pub struct RedundantClone;

impl Rule for RedundantClone {
    fn name(&self) -> &'static str {
        "redundant-clone"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = CloneVisitor {
            hits: Vec::new(),
            closure_depth: 0,
            async_depth: 0,
        };
        visitor.visit_file(file);

        // Escalation thresholds — raised from 5/10 to 15/30 after
        // dogfood showed framework-mandated clones dominating scores
        // in Axum routes and tower-lsp handlers.
        let count = visitor.hits.len();
        if count > 30 {
            for d in &mut visitor.hits {
                d.severity = Severity::Slop;
                d.weight = 1.5;
            }
        } else if count > 15 {
            for d in &mut visitor.hits {
                d.severity = Severity::Warning;
                d.weight = 1.0;
            }
        }

        visitor.hits
    }
}

struct CloneVisitor {
    hits: Vec<Diagnostic>,
    closure_depth: usize,
    async_depth: usize,
}

impl<'ast> Visit<'ast> for CloneVisitor {
    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.closure_depth += 1;
        syn::visit::visit_expr_closure(self, node);
        self.closure_depth -= 1;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.asyncness.is_some() {
            self.async_depth += 1;
        }
        syn::visit::visit_item_fn(self, node);
        if node.sig.asyncness.is_some() {
            self.async_depth -= 1;
        }
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if node.sig.asyncness.is_some() {
            self.async_depth += 1;
        }
        syn::visit::visit_impl_item_fn(self, node);
        if node.sig.asyncness.is_some() {
            self.async_depth -= 1;
        }
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "clone" && node.args.is_empty() {
            let suppress = (self.closure_depth > 0 && is_field_access_clone(node))
                // Clone on a method call return: foo().clone() — often framework pattern
                || is_method_return_clone(node)
                // Inside async fn body with clone on a variable — likely moving into a future
                || (self.async_depth > 0 && is_variable_clone(node));

            if !suppress {
                self.hits.push(Diagnostic {
                    rule: "redundant-clone",
                    message: "`.clone()` — can this borrow instead?".to_string(),
                    line: node.method.span().start().line,
                    severity: Severity::Hint,
                    weight: 0.5,
                });
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

fn is_field_access_clone(call: &syn::ExprMethodCall) -> bool {
    matches!(call.receiver.as_ref(), syn::Expr::Field(_))
}

fn is_method_return_clone(call: &syn::ExprMethodCall) -> bool {
    matches!(
        call.receiver.as_ref(),
        syn::Expr::MethodCall(_) | syn::Expr::Call(_)
    )
}

fn is_variable_clone(call: &syn::ExprMethodCall) -> bool {
    matches!(call.receiver.as_ref(), syn::Expr::Path(_))
}
