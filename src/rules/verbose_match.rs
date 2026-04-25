use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags `match` expressions that could be `if let`, `map`, or `unwrap_or`.
///
/// AI code generators default to full match blocks even when a combinator
/// or `if let` would be clearer:
/// ```ignore
/// match opt {
///     Some(v) => v,
///     None => default,
/// }
/// // vs: opt.unwrap_or(default)
/// ```
pub struct VerboseMatch;

impl Rule for VerboseMatch {
    fn name(&self) -> &'static str {
        "verbose-match"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = MatchVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct MatchVisitor {
    hits: Vec<Diagnostic>,
}

/// Returns true if the arm body is a simple expression (no blocks with
/// control flow, no `continue`, no `return`, no side-effecting statements).
fn is_simple_arm_body(expr: &syn::Expr) -> bool {
    match expr {
        // A block with multiple statements or control flow isn't "simple".
        syn::Expr::Block(block) => {
            block.block.stmts.len() <= 1
                && block.block.stmts.iter().all(|s| match s {
                    syn::Stmt::Expr(e, _) => is_simple_arm_body(e),
                    _ => false,
                })
        }
        // These indicate non-trivial error handling.
        syn::Expr::Return(_) | syn::Expr::Continue(_) | syn::Expr::Break(_) => false,
        // Macro calls (eprintln!, log::error!, etc.) are side effects.
        syn::Expr::Macro(_) => false,
        _ => true,
    }
}

impl<'ast> Visit<'ast> for MatchVisitor {
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        if node.arms.len() == 2 {
            // Only flag if both arms are simple expressions.
            let both_simple = node.arms.iter().all(|arm| is_simple_arm_body(&arm.body));
            if !both_simple {
                syn::visit::visit_expr_match(self, node);
                return;
            }

            let patterns: Vec<String> = node
                .arms
                .iter()
                .map(|arm| {
                    let pat = &arm.pat;
                    quote::quote!(#pat).to_string()
                })
                .collect();

            let pat_set: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();

            let is_option_match = pat_set.iter().any(|p| p.starts_with("Some"))
                && pat_set.contains(&"None");

            let is_result_match = pat_set.iter().any(|p| p.starts_with("Ok"))
                && pat_set.iter().any(|p| p.starts_with("Err"));

            let is_bool_match = (pat_set.contains(&"true") && pat_set.contains(&"false"))
                || (pat_set.contains(&"false") && pat_set.contains(&"true"));

            if is_option_match || is_result_match || is_bool_match {
                let suggestion = if is_option_match {
                    "consider `if let Some(v)`, `.map()`, or `.unwrap_or()`"
                } else if is_result_match {
                    "consider `if let Ok(v)`, `.map()`, or `?`"
                } else {
                    "consider `if`/`else` instead of matching on bool"
                };

                let line = node.match_token.span.start().line;
                self.hits.push(Diagnostic {
                    rule: "verbose-match",
                    message: format!("two-arm match could be simpler — {suggestion}"),
                    line,
                    severity: Severity::Warning,
                    weight: 1.0,
                });
            }
        }

        syn::visit::visit_expr_match(self, node);
    }
}
