use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags `.clone()` calls that look gratuitous.
///
/// AI code generators scatter `.clone()` everywhere to satisfy the borrow
/// checker instead of restructuring ownership. We flag every `.clone()` as
/// a hint and let density drive the score up.
///
/// Suppressed inside closures where `.clone()` extracts an owned value from
/// a borrowed reference (e.g. `|r| r.field.clone()`), since the clone is
/// mandatory there.
pub struct RedundantClone;

impl Rule for RedundantClone {
    fn name(&self) -> &'static str {
        "redundant-clone"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = CloneVisitor {
            hits: Vec::new(),
            closure_depth: 0,
        };
        visitor.visit_file(file);

        let count = visitor.hits.len();
        if count > 10 {
            for d in &mut visitor.hits {
                d.severity = Severity::Slop;
                d.weight = 2.0;
            }
        } else if count > 5 {
            for d in &mut visitor.hits {
                d.severity = Severity::Warning;
                d.weight = 1.5;
            }
        }

        visitor.hits
    }
}

struct CloneVisitor {
    hits: Vec<Diagnostic>,
    closure_depth: usize,
}

impl<'ast> Visit<'ast> for CloneVisitor {
    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.closure_depth += 1;
        syn::visit::visit_expr_closure(self, node);
        self.closure_depth -= 1;
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "clone" && node.args.is_empty() {
            // Inside a closure, .clone() on a field access is almost always
            // extracting an owned value from a borrowed reference — not gratuitous.
            let suppress = self.closure_depth > 0 && is_field_access_clone(node);

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

/// Check if the receiver of `.clone()` is a field access: `x.field.clone()`
/// or `x.field.sub.clone()`.
fn is_field_access_clone(call: &syn::ExprMethodCall) -> bool {
    matches!(call.receiver.as_ref(), syn::Expr::Field(_))
}
