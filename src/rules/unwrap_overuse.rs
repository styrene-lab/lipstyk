use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule, has_cfg_test_attr, has_test_attr};
use syn::visit::Visit;

/// Flags excessive `.unwrap()` and `.expect()` calls.
///
/// AI-generated Rust loves to unwrap everything instead of propagating
/// errors with `?`. Unwraps inside `#[test]` functions and `#[cfg(test)]`
/// modules are suppressed — panicking on failure is idiomatic in tests.
pub struct UnwrapOveruse;

impl Rule for UnwrapOveruse {
    fn name(&self) -> &'static str {
        "unwrap-overuse"
    }

    fn check(&self, file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = UnwrapVisitor {
            source: ctx.source,
            exclude_tests: ctx.exclude_tests,
            in_test: false,
            hits: Vec::new(),
        };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct UnwrapVisitor<'s> {
    source: &'s str,
    exclude_tests: bool,
    in_test: bool,
    hits: Vec<Diagnostic>,
}

impl<'s> Visit<'s> for UnwrapVisitor<'s> {
    fn visit_item_mod(&mut self, node: &'s syn::ItemMod) {
        let was_in_test = self.in_test;
        if has_cfg_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_item_mod(self, node);
        self.in_test = was_in_test;
    }

    fn visit_item_fn(&mut self, node: &'s syn::ItemFn) {
        let was_in_test = self.in_test;
        if has_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_item_fn(self, node);
        self.in_test = was_in_test;
    }

    fn visit_expr_method_call(&mut self, node: &'s syn::ExprMethodCall) {
        let method = node.method.to_string();
        if method == "unwrap" || method == "expect" {
            if self.in_test && self.exclude_tests {
                syn::visit::visit_expr_method_call(self, node);
                return;
            }

            let line = node.method.span().start().line;

            let source_line = self
                .source
                .lines()
                .nth(line.saturating_sub(1))
                .unwrap_or("");
            let unwrap_count =
                source_line.matches(".unwrap()").count() + source_line.matches(".expect(").count();

            let (severity, weight) = if self.in_test {
                // Test code: downweight heavily, hint only.
                (Severity::Hint, 0.1)
            } else if unwrap_count > 1 {
                (Severity::Slop, 3.0)
            } else {
                (Severity::Warning, 1.0)
            };

            self.hits.push(Diagnostic {
                rule: "unwrap-overuse",
                message: format!(
                    "`.{method}()` — consider propagating with `?` or handling the error"
                ),
                line,
                severity,
                weight,
            });
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}
